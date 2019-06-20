use std::env;
use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

use std::fs::File;
use std::io::ErrorKind;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, SystemExt};
use tokio::prelude::future::Future;
use tokio_timer::clock::Now;
use tokio_timer::{Delay, Interval};
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{
    BOOL, BYTE, DWORD, FALSE, HMODULE, LPARAM, LPCVOID, LPDWORD, LPVOID,
};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{
    GetModuleHandleA, GetProcAddress, LoadLibraryExA, DONT_RESOLVE_DLL_REFERENCES,
};
use winapi::um::memoryapi::{VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::processthreadsapi::{CreateRemoteThread, GetCurrentProcess, OpenProcess};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleBaseNameA, LIST_MODULES_ALL};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::{
    HANDLE, LPCSTR, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
    PROCESS_ALL_ACCESS,
};

fn get_process_id(name: &str) -> Result<DWORD, &str> {
    let mut system: sysinfo::System = sysinfo::System::new();
    system.refresh_all();
    let processes: Vec<&sysinfo::Process> = system.get_process_by_name(name);

    if processes.len() == 1 {
        return Ok(processes[0].pid() as DWORD);
    }
    return Err("Process not found!");
}

pub struct Process {
    handle: HANDLE,
}

impl Process {
    pub fn current() -> Self {
        return Process { handle: unsafe { GetCurrentProcess() } };
    }

    pub fn wait_for(name: &str) -> Self {
        let when = Instant::now() + Duration::from_millis(100);
        let task = Delay::new(when)
            .and_then(|_| {
                println!("Hello world!");
                Ok(())
            })
            .map_err(|e| panic!("delay errored; err={:?}", e));

        tokio::run(task);

        Process::by_name(name)
    }

    pub fn by_name(name: &str) -> Self {
        let pid: DWORD = get_process_id(name).unwrap();

        return Process { handle: unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) } };
    }

    pub fn alloc(&self, size: SIZE_T) -> Result<LPVOID, &str> {
        return Ok(unsafe {
            VirtualAllocEx(
                self.handle,
                NULL,
                size,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_EXECUTE_READWRITE,
            )
        });
    }

    pub fn free(&self, addr: LPVOID) -> Result<BOOL, &str> {
        return Ok(unsafe { VirtualFreeEx(self.handle, addr, 0, MEM_RELEASE) });
    }

    pub fn write_memory(&self, location: LPVOID, bytes: &[BYTE]) -> Result<LPVOID, &str> {
        let mut num_write: SIZE_T = 0;
        let success = unsafe {
            WriteProcessMemory(
                self.handle,
                location as LPVOID,
                bytes.as_ptr() as LPCVOID,
                bytes.len(),
                &mut num_write,
            )
        };
        if success == FALSE {
            return Err("Os error.");
        }
        if num_write == 0 {
            return Err("Wrote zero bytes.");
        }
        if num_write != bytes.len() {
            return Err("Wrote unexpected number of bytes.");
        }
        return Ok((location as LPARAM + num_write as LPARAM) as LPVOID);
    }

    pub fn write_dll_path(&self, path: &str) -> Result<LPVOID, &str> {
        let c_str = CString::new(path).unwrap();
        let bytes = c_str.as_bytes_with_nul();
        let addr = self.alloc(bytes.len()).unwrap();
        self.write_memory(addr, bytes);
        return Ok(addr);
    }

    pub fn create_remote_thread(
        &self, proc_addr: LPVOID, param_addr: LPVOID,
    ) -> Result<HANDLE, &str> {
        return Ok(unsafe {
            CreateRemoteThread(
                self.handle,
                NULL as LPSECURITY_ATTRIBUTES,
                0,
                Some(std::mem::transmute(proc_addr)),
                param_addr,
                0,
                NULL as LPDWORD,
            )
        });
    }

    pub fn load_dll(&self, path: &str) {
        let proc_addr = get_proc_address("kernel32.dll", "LoadLibraryA").unwrap();
        let dll_path_addr = self.write_dll_path(path).unwrap();

        let thread_handle: HANDLE = self.create_remote_thread(proc_addr, dll_path_addr).unwrap();
        unsafe { WaitForSingleObject(thread_handle, INFINITE) };
        self.free(dll_path_addr);
        unsafe { CloseHandle(thread_handle) };

        let path_c = CString::new(path).unwrap();
        unsafe { LoadLibraryExA(path_c.as_ptr() as LPCSTR, NULL, DONT_RESOLVE_DLL_REFERENCES) };
    }

    fn get_module(&self, name: &str) -> Result<HMODULE, String> {
        let mut modules: [HMODULE; 1024] = [0 as HMODULE; 1024];
        let mut num_modules: DWORD = 0;
        let success = unsafe {
            EnumProcessModulesEx(
                self.handle,
                modules.as_mut_ptr(),
                (modules.len() * size_of::<HMODULE>()) as DWORD,
                &mut num_modules,
                LIST_MODULES_ALL,
            )
        };
        if success == FALSE {
            return Err(String::from("Could not get module."));
        }

        for i in 0..(num_modules as SIZE_T) {
            let mut buffer: [c_char; 1024] = [0x00; 1024];
            let size = unsafe {
                GetModuleBaseNameA(
                    self.handle,
                    modules[i],
                    buffer.as_mut_ptr() as *mut winapi::ctypes::c_char,
                    buffer.len() as DWORD,
                )
            };

            if size > 0 {
                let module_name = unsafe { CStr::from_ptr(buffer.as_ptr()).to_str().unwrap() };
                if name == module_name {
                    return Ok(modules[i]);
                }
            }
        }
        return Err(format!("Module '{}' not found.", name));
    }

    pub fn exec(&self, module_name: &str, proc_name: &str) {
        let proc_addr = get_proc_address(module_name, proc_name).unwrap();
        let thread_handle: HANDLE = self.create_remote_thread(proc_addr, NULL).unwrap();
        //    unsafe { WaitForSingleObject(thread_handle, INFINITE) };
        unsafe { CloseHandle(thread_handle) };
    }

    pub fn close(&self) -> Result<BOOL, &str> {
        return Ok(unsafe { CloseHandle(self.handle) });
    }
}

pub fn get_module(name: &str) -> Result<HMODULE, String> {
    let c_str = CString::new(name).unwrap();
    let module_handle: HMODULE = unsafe { GetModuleHandleA(c_str.as_ptr() as LPCSTR) };
    if module_handle as LPARAM == 0 {
        return Err(format!("Module '{}' not found.", name));
    }
    return Ok(module_handle);
}

pub fn get_proc_address(module_name: &str, proc_name: &str) -> Result<LPVOID, &'static str> {
    let module = get_module(module_name).unwrap();
    let c_str = CString::new(proc_name).unwrap();
    return Ok(unsafe { GetProcAddress(module, c_str.as_ptr() as LPCSTR) } as LPVOID);
}

pub fn absolute_path<P>(path: P) -> PathBuf
where P: AsRef<Path> {
    let path = path.as_ref();
    if path.is_absolute() {
        return path.to_path_buf();
    }
    return env::current_dir().unwrap().join(path);
}

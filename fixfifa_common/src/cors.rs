use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::os::raw::c_char;
use std::mem;
use std::ptr;

use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, LPARAM, LPDWORD, LPVOID, LPCVOID, BOOL, BYTE};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{GetProcAddress, GetModuleHandleA};
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::processthreadsapi::{CreateRemoteThread, OpenProcess};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleBaseNameA, LIST_MODULES_ALL};
use winapi::um::winnt::{HANDLE, LPCSTR, PROCESS_ALL_ACCESS, MEM_RESERVE, MEM_COMMIT, PAGE_EXECUTE_READWRITE, MEM_RELEASE};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory, VirtualFreeEx, ReadProcessMemory};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use std::fmt::Debug;
use settings::Settings;
use sysinfo::{ProcessExt, SystemExt};

#[derive(Debug, Clone, Copy)]
pub struct ParamWrapper {
  pub ptr_param: LPARAM,
  pub ptr_result: LPARAM,
}

fn get_process_id(name: &str) -> Result<DWORD, &str> {
  let mut system: sysinfo::System = sysinfo::System::new();
  system.refresh_all();
  let processes: Vec<&sysinfo::Process> = system.get_process_by_name(name);

  if processes.len() == 1 {
    return Ok(processes[0].pid() as DWORD);
  }
  return Err("Process not found!");
}

pub struct CORProcess {
  pub handle: HANDLE,
}

impl Drop for CORProcess {
  fn drop(&mut self) {
    unsafe { CloseHandle(self.handle) };
  }
}

impl CORProcess {
  pub fn by_name(name: &str) -> Self {
    let pid: DWORD = get_process_id(name).unwrap();
    return CORProcess::new(pid);
  }

  pub fn new(pid: DWORD) -> Self {
    let handle = unsafe {
      OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid)
    };
    return CORProcess { handle };
  }

  fn get_module_handle(&self, module_name: &str) -> HMODULE {
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
      return 0 as HMODULE;
    }

    for i in 0..(num_modules as SIZE_T) {
      let mut buffer: [c_char; 1024] = [0x00; 1024];
      let size = unsafe {
        GetModuleBaseNameA(
          self.handle,
          modules[i],
          buffer.as_mut_ptr() as *mut _,
          buffer.len() as DWORD,
        )
      };

      if size > 0 {
        let _module_name = unsafe { CStr::from_ptr(buffer.as_ptr()).to_str().unwrap() };
        if module_name == _module_name {
          return modules[i];
        }
      }
    }
    return 0 as HMODULE;
  }

  fn get_proc_addr(&self, module_name: &str, proc_name: &str) -> LPARAM {
    let module_handle = self.get_module_handle(module_name);

    println!("{:?}", module_handle);

    let c_str = CString::new(proc_name).unwrap();

    let addr = unsafe{
      GetProcAddress(module_handle, c_str.as_ptr() as LPCSTR)
    };
    println!("{:?}", addr);

    return 0 as LPARAM;
//    println!("{:?}", "test");
//    return unsafe { GetProcAddress(module_handle, c_proc_name_ptr) } as LPARAM;
  }

  fn virtual_alloc<T: Sized + Copy>(&self) -> LPARAM {
    return unsafe {
      VirtualAllocEx(
        self.handle,
        NULL,
        mem::size_of::<T>(),
        MEM_RESERVE | MEM_COMMIT,
        PAGE_EXECUTE_READWRITE,
      )
    } as LPARAM;
  }

  fn virtual_free(&self, addr: LPARAM) -> BOOL {
    return unsafe {
      VirtualFreeEx(
        self.handle,
        addr as LPVOID,
        0,
        MEM_RELEASE,
      )
    };
  }

  fn read_proc_mem<T: Sized + Copy>(&self, addr: LPARAM) -> T {
    let mut buffer = std::vec::Vec::<BYTE>::with_capacity(std::mem::size_of::<T>());
    let ptr = buffer.as_mut_ptr() as *mut T;
    let mut num_read: SIZE_T = 0;
    let success = unsafe {
      ReadProcessMemory(
        self.handle,
        addr as LPCVOID,
        ptr as LPVOID,
        buffer.capacity(),
        &mut num_read,
      )
    };
    return unsafe { *ptr };
  }

  fn write_proc_mem<T: Sized + Copy>(&self, addr: LPARAM, bytes: &T) -> SIZE_T {
    let mut num_write: SIZE_T = 0;
    let success = unsafe {
      WriteProcessMemory(
        self.handle,
        addr as LPVOID,
        bytes as *const _ as LPCVOID,
        mem::size_of::<T>(),
        &mut num_write,
      )
    };
    return num_write;
  }

  fn create_remote_thread(&self, proc_addr: LPARAM, param_addr: LPARAM) -> HANDLE {
    return unsafe {
      CreateRemoteThread(
        self.handle,
        NULL as LPSECURITY_ATTRIBUTES,
        0,
        mem::transmute(proc_addr),
        param_addr as LPVOID,
        0,
        NULL as LPDWORD,
      )
    };
  }

  pub fn exec<T: Sized + Copy, R: Sized + Copy>(&self, module_name: &str, proc_name: &str, param: &T) -> R {
    let module_handle = self.get_module_handle(module_name) as LPARAM;

    let _module_handle = get_module(module_name).unwrap() as LPARAM;
    let _proc_addr = get_proc_address(module_name, proc_name).unwrap() as LPARAM;

    if module_handle == 0 {
      panic!("module not found");
    }

    let proc_addr = _proc_addr - _module_handle + module_handle;
    println!("{:X}: {:X} - {:X} + {:X}", proc_addr, _proc_addr, _module_handle, module_handle);

    let wrapper = ParamWrapper {
      ptr_param: self.virtual_alloc::<T>(),
      ptr_result: self.virtual_alloc::<R>(),
    };
    self.write_proc_mem(wrapper.ptr_param, param);

    let wrapper_addr = self.virtual_alloc::<ParamWrapper>();
    self.write_proc_mem(wrapper_addr, &wrapper);

    println!("{:X}: {:X} {:X}", wrapper_addr, wrapper.ptr_param, wrapper.ptr_result);

    let thread_handle = self.create_remote_thread(proc_addr, wrapper_addr);
    unsafe { WaitForSingleObject(thread_handle, INFINITE) };

    let result = self.read_proc_mem::<R>(wrapper.ptr_result);

    self.virtual_free(wrapper.ptr_param);
    self.virtual_free(wrapper.ptr_result);
    self.virtual_free(wrapper_addr);

    return result;
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

/**
 *  RemoteThread for unwrap / wrap / write params and results on DLL-Site.
 */
pub struct RemoteThread {
  pub wrapper: ParamWrapper,
}

impl RemoteThread {
  pub fn new(wrapper: &ParamWrapper) -> Self {
    let remote_thread = RemoteThread {
      wrapper: wrapper.clone(),
    };

    println!("{:?}", remote_thread.wrapper);

    return remote_thread;
  }

  pub fn read_param<T: Sized + Copy>(&self) -> T {
    let ptr = self.wrapper.ptr_param as *const T;
    return unsafe { *ptr };
  }

  pub fn write_result<R: Sized + Copy>(&self, result: &R) {
    unsafe {
      ptr::copy(
        result as *const R,
        self.wrapper.ptr_result as *mut R,
        mem::size_of_val(result),
      )
    };
  }
}

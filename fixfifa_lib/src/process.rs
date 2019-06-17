use std::cmp;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::mem::size_of;
use std::ptr::null_mut;

use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{
    BYTE, DWORD, FALSE, HIBYTE, HIWORD, HMODULE, LOBYTE, LOWORD, LPARAM, LPCVOID, LPVOID, MAKELONG,
    MAKEWORD, WORD, WPARAM,
};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{GetModuleInformation, MODULEINFO};
use winapi::um::winnt::{HANDLE, LPCSTR};

#[derive(Debug)]
pub struct ProcessError {
    msg: String,
}

impl From<io::Error> for ProcessError {
    fn from(error: io::Error) -> Self {
        return ProcessError { msg: error.to_string() };
    }
}

impl From<&str> for ProcessError {
    fn from(msg: &str) -> Self {
        return ProcessError { msg: msg.to_string() };
    }
}

impl From<String> for ProcessError {
    fn from(msg: String) -> Self {
        return ProcessError { msg };
    }
}

impl ProcessError {
    pub fn new<E>(error: E) -> ProcessError
    where E: Into<ProcessError> {
        return error.into();
    }

    #[inline]
    #[allow(non_snake_case)]
    pub fn NotYetImplemented(method: &str) -> ProcessError {
        return ProcessError::new(format!("Not Yet Implemented: {}.", method));
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.msg);
    }
}

impl Error for ProcessError {
    fn description(&self) -> &str {
        return self.msg.as_str();
    }
}

pub type Result<T> = std::result::Result<T, ProcessError>;

const RELATIVE_ADDRESS_T: SIZE_T = 4;
pub const DWORD_T: SIZE_T = 4;
pub const QWORD_T: SIZE_T = DWORD_T * 2;
const BUFFER_T: SIZE_T = 8192;

fn to_lparam(bytes: &[BYTE]) -> Result<LPARAM> {
    if bytes.len() != RELATIVE_ADDRESS_T {
        return Err(ProcessError::new("Invalid address size."));
    }
    return Ok(MAKELONG(MAKEWORD(bytes[0], bytes[1]), MAKEWORD(bytes[2], bytes[3])) as LPARAM);
}

fn to_dword_bytes(addr: WPARAM) -> Result<[BYTE; DWORD_T]> {
    //  let bytes: *const BYTE = addr as *const BYTE;
    //  let mut buffer: [BYTE; DWORD_T] = [0x00; DWORD_T];
    //  unsafe { bytes.copy_to(buffer.as_mut_ptr(), DWORD_T) };
    //  return Ok(buffer);
    let hi: WORD = HIWORD(addr as DWORD);
    let lo: WORD = LOWORD(addr as DWORD);
    return Ok([LOBYTE(lo), HIBYTE(lo), LOBYTE(hi), HIBYTE(hi)]);
}

fn to_qword_bytes(addr: WPARAM) -> Result<[BYTE; QWORD_T]> {
    //  let bytes: *const BYTE = addr as *const BYTE;
    //  let mut buffer: [BYTE; QWORD_T] = [0x00; QWORD_T];
    //  unsafe { bytes.copy_to(buffer.as_mut_ptr(), QWORD_T) };
    //  return Ok(buffer);
    let hi = to_dword_bytes((addr & 0xFFFFFFFF00000000) >> 32).unwrap();
    let lo = to_dword_bytes(addr & 0x00000000FFFFFFFF).unwrap();
    return Ok([lo[0], lo[1], lo[2], lo[3], hi[0], hi[1], hi[2], hi[3]]);
}

pub struct Process {
    handle: HANDLE,
}

pub struct ModuleInfo {
    pub start: LPARAM,
    pub end: LPARAM,
    pub entry: LPARAM,
    pub size: SIZE_T,
}

impl Process {
    pub fn new() -> Self {
        return Process { handle: unsafe { GetCurrentProcess() } };
    }

    pub fn read_memory(&self, address: LPARAM, to_read: SIZE_T) -> Result<Vec<BYTE>> {
        let mut buffer: [BYTE; BUFFER_T] = [0; BUFFER_T];
        let mut num_read: SIZE_T = 0;
        let success = unsafe {
            ReadProcessMemory(
                self.handle,
                address as LPCVOID,
                buffer.as_mut_ptr() as LPVOID,
                to_read,
                &mut num_read,
            )
        };
        if success == FALSE {
            return Err(ProcessError::new(io::Error::last_os_error()));
        }
        if num_read == 0 {
            return Err(ProcessError::new("Read zero bytes."));
        }
        return Ok(Vec::from(&buffer[0..num_read]));
    }

    pub fn read_addr(&self, address: LPARAM) -> Result<LPARAM> {
        let bytes: Vec<BYTE> = self.read_memory(address, RELATIVE_ADDRESS_T).unwrap();
        if bytes.len() != RELATIVE_ADDRESS_T {
            return Err(ProcessError::new("Invalid address size."));
        }
        return Ok(to_lparam(bytes.as_slice()).unwrap());
    }

    pub fn read_rel_target(&self, address: LPARAM, op_size: SIZE_T) -> Result<LPARAM> {
        // Relative target address is always 4 bytes.
        let rel_addr = self.read_addr(address + (op_size - 4) as LPARAM).unwrap();
        return Ok(address + op_size as LPARAM + rel_addr);
    }

    pub fn write_memory(&self, address: LPARAM, bytes: &[BYTE]) -> Result<LPARAM> {
        println!("{:X}: {} bytes", address, bytes.len());
        let mut num_write: SIZE_T = 0;
        let success = unsafe {
            WriteProcessMemory(
                self.handle,
                address as LPVOID,
                bytes.as_ptr() as LPCVOID,
                bytes.len(),
                &mut num_write,
            )
        };
        if success == FALSE {
            return Err(ProcessError::new(io::Error::last_os_error()));
        }
        if num_write == 0 {
            return Err(ProcessError::new("Wrote zero bytes."));
        }
        if num_write != bytes.len() {
            return Err(ProcessError::new("Wrote unexpected number of bytes."));
        }
        return Ok(address + num_write as LPARAM);
    }

    pub fn write_byte(&self, address: LPARAM, byte: BYTE) -> Result<LPARAM> {
        return self.write_memory(address, &[byte]);
    }

    pub fn write_addr(&self, address: LPARAM, addr: LPARAM, size: SIZE_T) -> Result<LPARAM> {
        if size == DWORD_T {
            return self.write_memory(address, &to_dword_bytes(addr as WPARAM).unwrap());
        } else if size == QWORD_T {
            return self.write_memory(address, &to_qword_bytes(addr as WPARAM).unwrap());
        }
        return Err(ProcessError::new("Invalid address size."));
        //    let addr_bytes: [BYTE; RELATIVE_ADDRESS_T] = to_bytes(addr);
        //    println!(
        //      "{:X}: {:X} -> {:X} {:X} {:X} {:X}",
        //      address,
        //      addr as LPARAM,
        //      addr_bytes[0],
        //      addr_bytes[1],
        //      addr_bytes[2],
        //      addr_bytes[3]
        //    );
    }

    pub fn write_rel_target(&self, address: LPARAM, op: &[BYTE], target: LPARAM) -> Result<LPARAM> {
        // Relative target address is always 4 bytes.
        let rel_addr: LPARAM = target - address - ((op.len() + RELATIVE_ADDRESS_T) as LPARAM);
        let target_address = self.write_memory(address, op).unwrap();
        return self.write_addr(target_address, rel_addr, DWORD_T);
    }

    pub fn find_pattern_mask(
        &self, start: LPARAM, end: LPARAM, find: &[BYTE], mask: &[BYTE],
    ) -> Result<LPARAM> {
        let mut chunk: LPARAM = start;

        while chunk < end {
            // Only read till end (including)
            let to_read = cmp::min(8192, cmp::max(0, end - chunk - 1)) as SIZE_T;
            if to_read < find.len() {
                // Nothing found.
                return Err(ProcessError::new("Pattern not found."));
            }

            let bytes: Vec<BYTE> = self.read_memory(chunk, to_read).unwrap();
            let num_read = bytes.len();

            for i in 0..(num_read - find.len() + 1) {
                let mut found = true;
                for j in 0..find.len() {
                    if bytes[i + j] != find[j] && mask[j] != 0x78 {
                        found = false;
                        break;
                    }
                }
                if found {
                    return Ok(chunk + i as LPARAM);
                }
            }

            chunk += num_read as LPARAM;
            if chunk < end {
                // Overlap search window but move at least one byte further.
                chunk -= cmp::min(num_read - 1, find.len() - 1) as LPARAM;
            }
        }

        return Err(ProcessError::new("Pattern not found."));
    }

    pub fn find_pattern(&self, start: LPARAM, end: LPARAM, find: &[BYTE]) -> Result<LPARAM> {
        let mask = vec![0x2D; find.len()];
        return self.find_pattern_mask(start, end, find, mask.as_slice());
    }

    pub fn find_space(&self, start: LPARAM, end: LPARAM, len: SIZE_T) -> Result<LPARAM> {
        let find = vec![0xCC; len];
        return self.find_pattern(start, end, find.as_slice());
    }

    pub fn get_module(&self, name: &str) -> Result<HMODULE> {
        let c_str = CString::new(name).unwrap();
        let module_handle: HMODULE = unsafe { GetModuleHandleA(c_str.as_ptr() as LPCSTR) };
        if module_handle as LPARAM == 0 {
            return Err(ProcessError::new("Module not found."));
        }
        return Ok(module_handle);
    }

    pub fn get_module_info(&self, name: &str) -> Result<ModuleInfo> {
        let module_handle = self.get_module(name).unwrap();
        let mut module_info: MODULEINFO =
            MODULEINFO { lpBaseOfDll: null_mut(), SizeOfImage: 0, EntryPoint: null_mut() };
        let success = unsafe {
            GetModuleInformation(
                self.handle,
                module_handle,
                &mut module_info,
                size_of::<MODULEINFO>() as DWORD,
            )
        };
        if success == FALSE {
            return Err(ProcessError::new(io::Error::last_os_error()));
        }
        let start = module_handle as LPARAM;
        let end = start + module_info.SizeOfImage as LPARAM;
        let entry = module_info.EntryPoint as LPARAM;
        let size = module_info.SizeOfImage as SIZE_T;

        return Ok(ModuleInfo { start, end, entry, size });
    }
}

use std::mem;
use std::ptr::null_mut;
use std::str;

use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{FALSE, LPARAM};
use winapi::shared::windef::HWND;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::GetConsoleWindow;
use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_SHOW};

use crate::pattern;
use crate::process;
use process::Result;
use process::{ModuleInfo, Process};
use process::{ProcessError, QWORD_T};

const EVENT_HOOK_T: SIZE_T = 76;
const EVENT_HOOK_OFFSET_T: SIZE_T = 18;

type LeaveGame = extern "C" fn() -> ();
static mut LEAVE_GAME_ADDR: LPARAM = 0;

pub fn leave_game() {
    unsafe {
        if LEAVE_GAME_ADDR != 0 {
            let le: LeaveGame = mem::transmute(LEAVE_GAME_ADDR);
            le();
        }
    }
}

pub fn search_leave_game(process: &Process) -> Result<()> {
    let module_info: ModuleInfo = process.get_module_info("fifa19.exe").unwrap();
    let mut current: LPARAM = module_info.start;

    let offset = 23;
    while current < module_info.end {
        current = process
            .find_pattern_mask(current, module_info.end, &pattern::LEAVE, &pattern::LEAVE_MASK)
            .unwrap();
        let addr = process.read_rel_target(current + offset, 7).unwrap();
        let addr_bytes = process.read_memory(addr, 9).unwrap();
        let addr_value = str::from_utf8(&addr_bytes).unwrap();

        if addr_value == "LeaveGame" {
            unsafe {
                LEAVE_GAME_ADDR = current;
            }
            break;
        }
        current = current + pattern::LEAVE_T as LPARAM;
    }

    return Ok(());
}

pub fn enable_event_hook(process: &Process, on_event: LPARAM) -> Result<()> {
    let module_info: ModuleInfo = process.get_module_info("fifa19.exe").unwrap();
    let pattern_addr: LPARAM =
        process.find_pattern(module_info.start, module_info.end, &pattern::EVENT).unwrap();
    let inject_addr: LPARAM =
        process.find_space(pattern_addr, module_info.end, EVENT_HOOK_T).unwrap();
    let hook_addr: LPARAM = pattern_addr + EVENT_HOOK_OFFSET_T as LPARAM;
    let return_addr: LPARAM = process.write_rel_target(hook_addr, &[0xE9], inject_addr).unwrap();
    let mut addr: LPARAM = inject_addr;
    addr = process.write_memory(addr, &pattern::PUSH).unwrap();
    addr = process.write_memory(addr, &pattern::PREPARE_CALL).unwrap();
    addr = process.write_addr(addr, on_event, QWORD_T).unwrap();
    addr = process.write_memory(addr, &pattern::CALL).unwrap();
    addr = process.write_memory(addr, &pattern::POP).unwrap();
    addr = process.write_memory(addr, &pattern::ORIGINAL).unwrap();
    addr = process.write_rel_target(addr, &[0xE9], return_addr).unwrap();
    println!("{:X}", pattern_addr);
    println!("{:X}", inject_addr);
    println!("{:X}", hook_addr);
    println!("{:X}", return_addr);
    //  process.write_memory(hook_addr, &pattern::ORIGINAL);
    return Ok(());
}

pub fn disable_quit_on_loose_focus(process: &Process) -> Result<()> {
    let module_info: ModuleInfo = process.get_module_info("fifa19.exe").unwrap();
    let mut pattern_addr: LPARAM = 0 as LPARAM;
    let mut current: LPARAM = module_info.start;

    let offset_je = 21;
    let offset = offset_je + 12;
    while current < module_info.end {
        current = process
            .find_pattern_mask(current, module_info.end, &pattern::ALT_TAB, &pattern::ALT_TAB_MASK)
            .unwrap();
        let addr = process.read_rel_target(current + offset, 7).unwrap();
        let addr_bytes = process.read_memory(addr, 39).unwrap();
        let addr_value = str::from_utf8(&addr_bytes).unwrap();

        println!("{:?}", addr_bytes);
        println!("{:?}", addr_value);

        if addr_value == "FIFA_SETTING_ENABLE_ALT_TAB_DISCONNECTS" {
            pattern_addr = current;
            break;
        }
        current = current + pattern::ALT_TAB_T as LPARAM;
    }

    if pattern_addr != 0 {
        let target_addr = process.read_rel_target(pattern_addr + offset_je, 6).unwrap();
        let addr_nop =
            process.write_rel_target(pattern_addr + offset_je, &[0xE9], target_addr).unwrap();
        process.write_memory(addr_nop, &[0x90]);
    }

    return Ok(());
}

fn alloc_console() -> Result<()> {
    let success = unsafe { AllocConsole() };
    if success == FALSE {
        return Err(ProcessError::new("Failed to alloc console."));
    }
    return Ok(());
}

fn get_console_window(alloc: bool) -> Result<HWND> {
    let window = unsafe { GetConsoleWindow() };
    if window == null_mut() {
        if !alloc {
            return Err(ProcessError::new("Console window not found."));
        }
        alloc_console();
        return get_console_window(!alloc);
    }
    return Ok(window);
}

fn console(enabled: bool) -> Result<()> {
    let window = get_console_window(enabled).unwrap();
    let success = unsafe { ShowWindow(window, if enabled { SW_SHOW } else { SW_HIDE }) };
    if success == FALSE {
        return Err(ProcessError::new("ShowWindow failed."));
    }
    return Ok(());
}

pub fn enable_console() -> Result<()> {
    return console(true);
}

pub fn disable_console() -> Result<()> {
    return console(false);
}

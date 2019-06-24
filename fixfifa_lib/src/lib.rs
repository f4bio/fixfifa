#![cfg(windows)]
#[macro_use]
extern crate log;
extern crate log4rs;

use crate::state::State;
use std::ffi::CStr;
use std::os::raw::c_char;
use winapi::shared::minwindef::{BOOL, DWORD, HMODULE, LPARAM, LPVOID, TRUE};

mod fifa19;
mod pattern;
mod process;
mod state;

const DLL_PROCESS_DETACH: DWORD = 0;
const DLL_PROCESS_ATTACH: DWORD = 1;
// const DLL_THREAD_ATTACH: DWORD = 2;
// const DLL_THREAD_DETACH: DWORD = 3;
static mut H_INST_DLL: HMODULE = 0 as HMODULE;
static mut GAME_STATE: Option<State> = None;

// const GLOBAL_DATA: GlobalData = GlobalData{
//  bad_clubs: ,
//  version_name: "".to_string(),
//  version_code: 0
//};
// const GROUP_ATTRIBUTE: GroupAttribute = GroupAttribute{
//  i: "".to_string(),
//  o: "".to_string(),
//  n: "".to_string(),
//  z: "".to_string(),
//  t: "".to_string(),
//  j: "".to_string(),
//  c: "".to_string()
//};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(hInstDLL: HMODULE, fdwReason: DWORD, _lpvReserved: LPVOID) -> BOOL {
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            unsafe { H_INST_DLL = hInstDLL };
            return dll_init().unwrap();
        }
        DLL_PROCESS_DETACH => TRUE,
        _ => TRUE,
    };
    return TRUE;
}

// http://mgattozzi.com/global-uninitialized/
/// Convenience function to access the variable inside CTX_1
unsafe fn game_state() -> &'static mut State {
    match GAME_STATE {
        Some(ref mut x) => &mut *x,
        None => panic!(),
    }
}

fn dll_init() -> process::Result<BOOL> {
    //  msgbox::create("Title", "Process attached", IconType::INFO);
    fifa19::enable_console();
    println!("dll_init!");
    let fifa19_process = process::Process::new();
    fifa19::search_leave_game(&fifa19_process);
    fifa19::enable_event_hook(&fifa19_process, on_event as LPARAM);
    fifa19::disable_quit_on_loose_focus(&fifa19_process);

    //  println!("lib {:X}", on_event as LPARAM);
    // get_list();
    //  return Err(process::ProcessError::new("debug"));
    return Ok(TRUE);
}

fn to_string(char_ptr: *const c_char) -> String {
    unsafe {
        return CStr::from_ptr(char_ptr).to_string_lossy().to_string();
    }
}

fn check_club(event_data: String) {
    let parts: Vec<&str> = event_data.split_terminator(';').collect();
    let mut parts_iter = parts.into_iter();

    let i = parts_iter.find(|&p| p.starts_with("I_")).unwrap();

    println!("check club with id={}", i);
}

fn check_arbitrator(event_data: String) {
    //    let parts: Vec<&str> = event_data.split_terminator(';').collect();
    //    let mut parts_iter = parts.into_iter();

    println!("parsing event: {}", event_data);
}

fn parse_event(state: &State, event_data: &String) {
    //  let parsed_data = GroupAttribute::from_str(data.as_str());
    println!("parsing event: {} / {}", state, event_data);
}

#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn on_event(event_name_ptr: *const c_char, event_data_ptr: *const c_char) {
    let event_name: String = to_string(event_name_ptr);
    let event_data: String = to_string(event_data_ptr);

    info!("got event: {} / {}", event_name, event_data);
    //  println!("got event: {} / {}", event_name, event_data);
    //  fifa19::leave_game();

    //  unsafe {
    //    let mut game_state = match GAME_STATE {
    //      Some(ref mut x) => &mut *x,
    //      None => panic!(),
    //    };
    //    parse_event(&game_state, &event_data)
    //  }
    //
    // TODO: this: (its broken; game crashes)
    //
    //  match event_name.as_str() {
    //    "EVENT_PREGAME_GROUP_ATTRIBUTE_CHANGED" => check_club(event_data),
    //    "EVENT_PREGAME_ARBITRATOR_IS_READY" => check_arbitrator(event_data),
    //    _ => println!(
    //      "dont know what to do with event named '{}' with data '{}'",
    //      event_name, event_data
    //    ),
    //  };
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn init() {
    println!("extern init call");
    //  unsafe {
    //    println!(
    //      "loaded global data! (version: {:?})",
    //      serde_json::to_string(&GLOBAL_DATA).unwrap()
    //    );
    //  }
}

#[macro_use]
extern crate log;
extern crate fixfifa_common;
extern crate fixfifa_ui;
extern crate log4rs;
extern crate sysinfo;
extern crate winapi;

use fixfifa_common::cors::CORProcess;
use std::path::{Path, PathBuf};
use winapi::um::winnt::{HANDLE, LPCSTR};
use winapi::shared::minwindef::{DWORD, LPARAM};
use fixfifa_common::settings::Settings;
use std::ffi::CString;
use winapi::um::libloaderapi::{LoadLibraryExA, DONT_RESOLVE_DLL_REFERENCES};
use winapi::shared::ntdef::NULL;

mod injector;

fn main() {
  let cp = CORProcess::new(injector::Process::by_name("FIFA19.exe").pid as DWORD);

  let lib_dll_path = Path::new(".").join("target").join("debug").join("fixfifa.dll");
  let path_c = CString::new(lib_dll_path.canonicalize().unwrap().to_str().unwrap()).unwrap();
  unsafe { LoadLibraryExA(path_c.as_ptr() as LPCSTR, NULL, DONT_RESOLVE_DLL_REFERENCES) };

  let settings = Settings {
    alt_tab: false,
    blacklist: true,
    skip_launcher: false,
    skip_language_selection: true
  };

  let result = cp.exec::<Settings,Settings>(
    "fixfifa.dll",
    "settings",
    &settings,
  );
  println!("{} {} {} {}", result.alt_tab, result.blacklist, result.skip_launcher, result.skip_language_selection);
//    let log_config_path = Path::new(".").join("config").join("log4rs.yaml");
//
//    let lib_dll_path = Path::new(".").join("target").join("debug").join("fixfifa.dll");
//
//    println!("using log config: '{}'", log_config_path.to_str().unwrap());
//    log4rs::init_file(log_config_path.to_str().unwrap(), Default::default()).unwrap();
//
//    println!("using dll: '{}'", lib_dll_path.canonicalize().unwrap().to_str().unwrap());
//
//    //    fixfifa_ui::start_ui();
//
//    //  attach to process
//    match injector::Process::by_name("FIFA19.exe") {
//        process => {
//            // inject dll (DLLMain called)
//            process.load_dll(lib_dll_path.canonicalize().unwrap().to_str().unwrap());
//
//            // call additional init fn
//            process.exec("fixfifa.dll", "init");
//
//            // close process handle
//            process.close();
//
//            let cp = CORProcess::new(process.pid);
//            let settings = Settings {
//              alt_tab: true,
//              blacklist: true,
//              skip_launcher: true,
//              skip_language_selection: false
//            };
//            let mut applied = false;
//            cp.exec(
//              "fixfifa.dll",
//              "settings",
//              &settings,
//              &mut applied,
//            );
//            println!("{:?}", applied);
//        }
//    }
}

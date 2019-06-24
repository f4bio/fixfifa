#[macro_use]
extern crate log;
extern crate fixfifa_common;
extern crate fixfifa_ui;
extern crate log4rs;
extern crate sysinfo;
extern crate tokio;
extern crate winapi;

use fixfifa_common::settings::Settings;
use std::path::{Path, PathBuf};

mod injector;

fn main() {
    //    let settings = Settings::new();
    let log_config_path = Path::new(".").join("config").join("log4rs.yaml");
    let lib_dll_path = Path::new(".").join("target").join("debug").join("fixfifa.dll");

    println!("using log config: '{}'", log_config_path.to_str().unwrap());
    log4rs::init_file(log_config_path.to_str().unwrap(), Default::default()).unwrap();

    println!("using dll: '{}'", lib_dll_path.canonicalize().unwrap().to_str().unwrap());
    //    fixfifa_ui::start_ui();

    //  attach to process
    match injector::Process::wait_for("FIFA19.exe") {
        process => {
            // inject dll (DLLMain called)
            process.load_dll(lib_dll_path.canonicalize().unwrap().to_str().unwrap());

            // call additional init fn
            process.exec("fixfifa.dll", "init");

            // close process handle
            process.close();
        }
    }
}

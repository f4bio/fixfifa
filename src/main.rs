#![cfg(windows)]
extern crate log4rs;
extern crate sysinfo;
extern crate winapi;
extern crate fixfifa_ui;
extern crate fixfifa_common;

use std::path::Path;

mod injector;

fn main() {
  let log_config_path = Path::new(".")
    .join("config")
    .join("log4rs.yml");

  let lib_dll_path = Path::new(".")
    .join("target")
    .join("debug")
    .join("fixfifa.dll");

  log4rs::init_file(log_config_path.to_str().unwrap(), Default::default()).unwrap();

  // get absolute path
  let dll_path = injector::absolute_path(lib_dll_path.to_str().unwrap());

  //  attach to process
  //  match injector::Process::by_name("FIFA19.exe") {
  match injector::Process::by_name("cmd.exe") {
    process => {
      // inject dll (DLLMain called)
      process.load_dll(dll_path.to_str().unwrap());

      // call additional init fn
      process.exec("fixfifa.dll", "init");

      // close process handle
      process.close();

      fixfifa_ui::start_ui();
    }
  }
}

#![cfg(windows)]
extern crate log4rs;
extern crate sysinfo;
extern crate winapi;
extern crate fixfifa_ui;
extern crate fixfifa_common;

mod injector;

fn main() {
  log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

  // get absolute path
  let dll_path = injector::absolute_path("target\\debug\\fixfifa.dll");

  //  attach to process
  match injector::Process::by_name("FIFA19.exe") {
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

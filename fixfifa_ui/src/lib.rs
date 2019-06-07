#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate fixfifa_common;
#[macro_use]
extern crate serde_derive;

use rocket::request::Form;
use rocket::request::{FlashMessage};
use rocket::response::{Flash, Redirect};
use rocket::{Request, State};
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::process;
use std::process::Command;
use std::ptr::null;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Weak};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use systray::Application;
use fixfifa_common::settings::{Setting, Settings};
use std::path::Path;

// TODO: use following statics instead of hardcoded paths
//Path::new("/etc").join("passwd")
//static UI_DIR: &'static [u8] = include_dir!("../fixfifa-ui/dist/");
//static LOGO_FILE: &'static [u8] = include_bytes!("../assets/tray_icon-256.ico");
//static DLL_FILE: &'static [u8] = include_bytes!("../target/debug/fixfifa.dll");
//static LOG_CONFIG_FILE: &'static str = include_str!("../config/log4rs.yaml");

#[derive(Debug, Serialize)]
struct Context<'a, 'b> {
  flash: Option<(&'a str, &'b str)>,
  settings: Settings,
}

impl<'a, 'b> Context<'a, 'b> {
  pub fn raw(msg: Option<(&'a str, &'b str)>) -> Context<'a, 'b> {
    Context {
      flash: msg,
      settings: Settings::get_all(),
    }
  }
}

#[post("/settings", data = "<settings_form>")]
fn set_all_settings(settings_form: Form<Settings>) -> Flash<Redirect> {
  let settings = settings_form.into_inner();
  let _updates = Settings {
    alt_tab: settings.alt_tab,
    blacklist: settings.blacklist,
    skip_launcher: settings.skip_launcher,
    skip_language_selection: settings.skip_language_selection,
  };

  Settings::set_all(_updates);
  return Flash::success(
    Redirect::to("/"),
    format!("settings applied: '{:?}'.", settings),
  );
}

#[post("/setting", data = "<setting_form>")]
fn set_single_setting(setting_form: Form<Setting<bool>>) -> Flash<Redirect> {
  let setting = setting_form.into_inner();

  Settings::set_single(&setting.key, setting.value);
  return Flash::success(
    Redirect::to("/"),
    format!("'{}' hack successfully activated.", &setting.key),
  );
}

#[get("/settings")]
fn get_all_settings() -> Template {
  Template::render("index", &Settings::get_all())
}

#[get("/setting/<key>")]
fn get_single_setting(key: String) -> Template {
  Template::render("index", &Settings::get_single(&key))
}

#[get("/")]
fn index(settings: State<Settings>, flash: Option<FlashMessage>) -> Template {
  let flash_message = match flash {
    Some(ref msg) => Some((msg.name(), msg.msg())),
    None => Some((("error"), ("unknown error"))),
  };

  Template::render(
    "index",
    Context {
      flash: flash_message,
      settings: Settings::get_all(),
    },
  )
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
  let mut map = HashMap::new();
  map.insert("path", req.uri().path());
  Template::render("error/404", &map)
}

fn start_tray(tx: Sender<u8>) -> JoinHandle<()> {
  println!("start_tray()");

  return thread::spawn(move || {
    match systray::Application::new() {
      Ok(w) => {
        // TODO: get host and port etc from rocket.rs config https://api.rocket.rs/v0.3/rocket/config/struct.Config.html
        let address: &str = "localhost";
        let port = 31337;
        let _url = format!("http://{}:{}", address, port);
        let mut app = w;
        let ico_path = Path::new(".")
          .join("assets")
          .join("tray_icon-256.ico");

        // icon
        app
          .set_icon_from_file(
            &String::from(ico_path.canonicalize().unwrap().to_str().unwrap())
          ).ok();
        // top most entry ("Open Settings")
        app
          .add_menu_item(&"Settings".to_string(), move |_| {
            println!("opening '{}'...", _url);
            Command::new("explorer ")
              .args(&[&_url])
              .output()
              .expect("failed to execute process");
          })
          .ok();

        app.add_menu_separator().ok();
        // last entry ("Quit")
        app
          .add_menu_item(&"Quit".to_string(), move |_app| {
            println!("quitting...");
            tx.send(0).unwrap();
            _app.quit();
          })
          .ok();
        app.wait_for_message();
      }
      Err(_) => panic!("Can't create Application daemon!"),
    };
  });
}

pub fn start_web() -> JoinHandle<()> {
  println!("start_web()");

  return thread::spawn(move || {
    rocket::ignite()
      .mount(
        "/",
        routes![
          index,
          set_all_settings,
          set_single_setting,
          get_all_settings,
          get_single_setting
        ],
      )
      .mount("/", StaticFiles::from("dist"))
      .attach(Template::fairing())
      .manage(Settings::new())
      .register(catchers![not_found])
      .launch();
  });
}

pub fn start_ui() {
  println!("start_ui...");

  let (tx, rx) = mpsc::channel();
  //  let tx1 = mpsc::Sender::clone(&tx);

  println!("creating threads...");
  let _web_thread = start_web();
  let _tray_thread = start_tray(tx);

  println!("Listing on channel...");

  for received in rx {
    println!("Got: {}", received);

    match received {
      0 => {
        println!("exit command received");
        process::exit(0x0);
      }
      _ => println!("unknown command '{}' received", received),
    }
  }
}

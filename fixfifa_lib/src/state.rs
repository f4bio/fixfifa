use crate::state::objects::club::Club;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use tempfile::tempfile;

mod objects;

const DEBUG_MODE: &'static bool = &true;
const DB_NAME: &'static str = "globals.db";
const GLOBAL_DATA_URL: &str =
  "https://s3.eu-central-1.amazonaws.com/fixfifa/globals.json";

#[derive(Deserialize, Serialize)]
pub struct GlobalData {
  bad_clubs: Vec<Club>,
  version_name: String,
  version_code: u16,
}

impl GlobalData {
  pub fn new() -> Self {
    let mut globals = PickleDb::new(
      DB_NAME,
      PickleDbDumpPolicy::DumpUponRequest,
      SerializationMethod::Json,
    );

    let url: Url = Url::parse(GLOBAL_DATA_URL).unwrap();
    // Create a file inside of `std::env::temp_dir()`.
    let mut file = tempfile().unwrap();
    let mut resp = reqwest::get(url).unwrap();
    let text_resp: String = resp.text().unwrap();
    file.write_all(text_resp.as_bytes()).unwrap();

    let deserialized: GlobalData =
      serde_json::from_str(text_resp.as_str()).unwrap();

    globals.set("version_name", &deserialized.version_name);
    globals.set("version_code", &deserialized.version_code);
    globals.lcreate("bad_clubs");

    let mut bad_clubs_iter = &deserialized.bad_clubs.iter();
    for bc in bad_clubs_iter.as_ref() {
      globals.ladd("bad_clubs", bc);
    }

    return deserialized;
  }

  pub fn bad_clubs(&self) -> &Vec<Club> {
    &self.bad_clubs
  }
  pub fn version_name(&self) -> &String {
    &self.version_name
  }
  pub fn version_code(&self) -> &u16 {
    &self.version_code
  }
}

#[derive(Deserialize, Serialize)]
pub struct State {
  team0: Club,
  team1: Club,
}

impl State {
  /// https://users.rust-lang.org/t/idiomatic-way-to-construct-object-with-some-non-required-fields/8078/4
  pub fn new() -> Self {
    // https://doc.rust-lang.org/std/string/struct.String.html#method.new
    State {
      team0: Club::new(),
      team1: Club::new(),
    }
  }
}

impl Display for State {
  fn fmt(&self, c: &mut Formatter) -> fmt::Result {
    write!(c, "team0={} team1={}", self.team0, self.team1,)
  }
}

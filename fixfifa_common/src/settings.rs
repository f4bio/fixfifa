use ini::ini::Properties;
use ini::Ini;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rocket::request::FromFormValue;
use sled::Db;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, FromForm, Debug)]
pub struct Setting<T> {
    pub key: String,
    pub value: T,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct InMemorySettings {
    pub alt_tab: bool,
    pub blacklist: bool,
}

#[derive(Serialize, Deserialize, FromForm, Debug)]
pub struct Settings {
    pub game_dir: String,
    pub skip_launcher: bool,
    pub skip_language_selection: bool,
    // TODO: use `InMemorySettings`
    pub alt_tab: bool,
    pub blacklist: bool,
}

const DEBUG_MODE: &'static bool = &false;
const DB_NAME: &'static str = "config.pickle.db";
// TODO: get from somewhere else
const ORIGIN_GAMES_DIRECTORY: &'static str = "D:\\Origin Games";

impl Settings {
    fn get_config_ini_path() -> PathBuf {
        return Path::new(&Settings::games_dir())
            .join("FIFA 19")
            .join("FIFASetup")
            .join("config.ini");
    }

    fn get_locale_ini_path() -> PathBuf {
        return Path::new(&Settings::games_dir()).join("FIFA 19").join("Data").join("locale.ini");
    }

    fn get_locale_ini_bak_path() -> PathBuf {
        return Path::new(&Settings::games_dir())
            .join("FIFA 19")
            .join("Data")
            .join("locale.ini.bak");
    }

    fn load_config_ini() -> Ini {
        Ini::load_from_file(Settings::get_config_ini_path()).unwrap()
    }

    fn load_locale_ini() -> Ini {
        // before loading `locale.ini`, comment lines at the end of the file (starting with `//`)
        // have to be removed, otherwise ini-parsing fails, and program exits
        // btw: those aren't allowed or correct, thanks EA...
        // https://en.wikipedia.org/wiki/INI_file#Escape_characters
        let _locale_data =
            fs::read_to_string(Settings::get_locale_ini_path()).expect("Unable to read file");

        let mut _locale_data_cleaned = String::new();
        for l in _locale_data.lines() {
            if l.starts_with("//") {
                continue;
            }
            _locale_data_cleaned.push_str(&l);
            _locale_data_cleaned.push_str("\n");
        }

        Ini::load_from_str(_locale_data_cleaned.as_str()).unwrap()
    }

    pub fn new() -> Settings {
        //        let sledDb = Db::start_default("config.sled.db").unwrap();
        let mut db =
            PickleDb::new(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json);

        // TODO: find out how to do this "correct"
        let games_dir: String = "D:\\Origin Games".to_string();
        //        let game_dir: String = match env::var_os("PROGRAMFILES(X86)") {
        //            Some(val) => Path::new(&val).join("Origin Games").to_str().unwrap().to_string(),
        //            None => Path::new("C:")
        //                .join("Program Files (x86)")
        //                .join("Origin Games")
        //                .to_str()
        //                .unwrap()
        //                .to_string(),
        //        };

        // needs to be done first:
        println!("games_dir: {}", games_dir);
        db.set("games_dir", &games_dir).unwrap();

        let _config: Ini = Settings::load_config_ini();
        let _locale: Ini = Settings::load_locale_ini();

        let auto_launch = _config.get_from_or(None::<String>, "AUTO_LAUNCH", "0");
        let use_language_select = _locale.get_from_or(Some("LOCALE"), "USE_LANGUAGE_SELECT", "0");

        let skip_launcher: bool = auto_launch.eq("0");
        let skip_language_selection: bool = use_language_select.eq("0");

        println!("auto_launch: {}", auto_launch);
        println!("use_language_select: {}", use_language_select);
        println!("skip_launcher: {}", skip_launcher);
        println!("skip_language_selection: {}", skip_language_selection);

        db.set("alt_tab", &true).unwrap();
        db.set("blacklist", &true).unwrap();
        db.set("skip_launcher", &skip_launcher).unwrap();
        db.set("skip_language_selection", &skip_language_selection).unwrap();

        Settings {
            game_dir: db.get::<String>("games_dir").unwrap(),
            skip_launcher: db.get::<bool>("skip_launcher").unwrap(),
            skip_language_selection: db.get::<bool>("skip_language_selection").unwrap(),
            alt_tab: db.get::<bool>("alt_tab").unwrap(),
            blacklist: db.get::<bool>("blacklist").unwrap(),
        }
    }

    pub fn get_all() -> Settings {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        Settings {
            game_dir: db.get::<String>("games_dir").unwrap(),
            skip_launcher: db.get::<bool>("skip_launcher").unwrap(),
            skip_language_selection: db.get::<bool>("skip_language_selection").unwrap(),
            alt_tab: db.get::<bool>("alt_tab").unwrap(),
            blacklist: db.get::<bool>("blacklist").unwrap(),
        }
    }

    pub fn games_dir() -> String {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<String>("games_dir").unwrap()
    }

    pub fn alt_tab() -> bool {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<bool>("alt_tab").unwrap()
    }

    pub fn blacklist() -> bool {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<bool>("blacklist").unwrap()
    }

    pub fn skip_launcher() -> bool {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<bool>("skip_launcher").unwrap()
    }

    pub fn skip_language_selection() -> bool {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<bool>("skip_language_selection").unwrap()
    }

    pub fn get_single(key: &str) -> bool {
        let db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.get::<bool>(key).unwrap()
    }

    pub fn set_all(settings: &Settings) {
        let mut db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.set("game_dir", &settings.game_dir).unwrap();
        db.set("alt_tab", &settings.alt_tab).unwrap();
        db.set("blacklist", &settings.blacklist).unwrap();
        db.set("skip_launcher", &settings.skip_launcher).unwrap();
        db.set("skip_language_selection", &settings.skip_language_selection).unwrap();
        Settings::apply();
    }

    pub fn set_single<'a>(key: &str, value: bool) -> () {
        let mut db =
            PickleDb::load(DB_NAME, PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)
                .unwrap();

        db.set::<bool>(key, &value).unwrap()
    }

    fn apply() {
        let settings = Settings::get_all();

        if settings.alt_tab {
            // TODO:
        }
        if settings.blacklist {
            // TODO:
        }
        if settings.skip_launcher {
            let mut _config: Ini = Settings::load_config_ini();
            let auto_launch: String = match settings.skip_launcher {
                true => String::from("1"),
                false => String::from("0"),
            };
            _config.set_to(Some(""), String::from("AUTO_LAUNCH"), auto_launch);
            _config.write_to_file(Settings::get_config_ini_path()).unwrap();
        }
        if settings.skip_language_selection {
            let mut _locale: Ini = Settings::load_locale_ini();
            let use_language_select: String = match settings.skip_language_selection {
                true => String::from("0"),
                false => String::from("1"),
            };
            _locale.set_to(
                Some("LOCALE"),
                String::from("USE_LANGUAGE_SELECT"),
                use_language_select,
            );
            _locale.write_to_file(Settings::get_locale_ini_path()).unwrap();
        }
    }
}

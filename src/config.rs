use std::{error::Error, fs, io, path::Path};

use clap::builder;
use crossterm::{event::KeyCode, ErrorKind};
use directories_next::ProjectDirs;
use serde::Deserialize;

static DEFAULT_CONFIG: &str = include_str!(r"..\default_config.json");

#[test]
fn test_config() -> Result<(), Box<dyn Error>> {
    let conf = Config::get();
    dbg!(conf);
    Ok(())
}

impl Config {
    pub fn get() -> Result<Self, io::Error> {
        let project_dir = ProjectDirs::from("", "", "tetris tui").ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Config file location could not be generated!",
        ))?;

        let conf_location = get_config_location(&project_dir);

        let json_string = fs::read_to_string(&conf_location).or_else(|e| {
            // if the file is not found, try to create it from the `DEFAULT_CONFIG`
            if let io::ErrorKind::NotFound = e.kind() {
                eprintln!("Could not find your config file, trying to create...");
                dbg!(e.kind());
                // create all directories
                fs::create_dir_all(project_dir.config_dir())?;
                // write the default file
                fs::write(&conf_location, DEFAULT_CONFIG)?;
                // read the new file
                fs::read_to_string(&conf_location)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Config file location not found!",
                ))
            }
        });

        // if the config file failed, use the default config
        let actual_config = match json_string {
            Err(_) => {
                eprintln!("Using Default Config!");
                DEFAULT_CONFIG
            }
            Ok(ref str) => &str,
        };

        let json: Config = serde_json::from_str(actual_config)?;

        Ok(json)
    }
}

const CONFIG_FILE_NAME: &str = "tetris.json";

/// join the project dirs' config dir with the CONFIG_FILE_NAME const and return as a String
fn get_config_location(project_dir: &ProjectDirs) -> String {
    let mut config_path_buf = project_dir.config_dir().to_path_buf();
    config_path_buf.push(CONFIG_FILE_NAME);
    config_path_buf.to_str().unwrap().to_owned()
}

/// needed because serde doesnt accept literals as default,
///
/// they have to result from a function
fn keycode_null() -> KeyCode {
    KeyCode::Null
}

/// Holds Config data,
/// optional Keys are the Null Variant.
///
/// defaults are provided
#[derive(Deserialize, Debug)]
pub struct Config {
    pub left: KeyWithAlt,
    pub right: KeyWithAlt,
    pub rotate: KeyWithAlt,
    pub soft_drop: KeyWithAlt,
    pub hard_drop: KeyWithAlt,

    // pause and quit
    #[serde(default = "keycode_null", with = "KeyCodeDef")]
    pub pause: KeyCode,
    #[serde(default = "keycode_null", with = "KeyCodeDef")]
    pub quit: KeyCode,
    // used in the pause menu
    #[serde(default = "keycode_null", with = "KeyCodeDef", rename = "continue")]
    pub continue_key: KeyCode,
}
/// mirrors a subset of the KeyCode definition
///
/// to be able to use Deserialize
/// for the crossterm `KeyCode`
#[derive(Debug, Deserialize)]
#[serde(remote = "KeyCode")]
pub enum KeyCodeDef {
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// A character.
    ///
    /// `KeyCode::Char('c')` represents `c` character, etc.
    #[serde(rename = "char_key")]
    Char(char),
    /// Null.
    Null,
    /// Escape key.
    Esc,
    Enter,
}

/// struct containing a KeyCode and maybe an alternative Key
#[derive(Debug, Deserialize)]
pub struct KeyWithAlt {
    #[serde(with = "KeyCodeDef")]
    pub key: KeyCode,
    #[serde(default = "keycode_null", with = "KeyCodeDef")]
    pub alt: KeyCode,
}

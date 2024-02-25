use crossterm::event::KeyCode;
use directories_next::ProjectDirs;
// use dirs::config_dir;

use serde::Deserialize;
use std::{fs, io};

static DEFAULT_CONFIG: &str = include_str!(r"../default_config.json");

impl Config {
    pub fn get() -> Config {
        return fallible_inner().unwrap_or_else(|e| {
            eprintln!("Using default Config! due to {e}");
            serde_json::from_str(DEFAULT_CONFIG).unwrap()
        });

        /// inner function to simplify Error Propagation
        fn fallible_inner() -> Result<Config, io::Error> {
            let project_dir = ProjectDirs::from("", "", "Tetris Tui").ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Config file location could not be generated!",
            ))?;

            let conf_location = get_config_location(&project_dir);

            let json_string = fs::read_to_string(&conf_location).or_else(|e| {
                // if the file is not found, try to create it from the `DEFAULT_CONFIG`
                if let io::ErrorKind::NotFound = e.kind() {
                    eprintln!("Could not find your config file, trying to create at {}", conf_location);
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
                    ))?
                }
            });

            // if the config file failed, use the default config
            serde_json::from_str::<Config>(&json_string?)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "JSON could not be parsed"))
        }
    }
}

const CONFIG_FILE_NAME: &str = "tetris.json";

/// join the project dirs' config dir with the CONFIG_FILE_NAME const and return as a String
pub fn get_config_location(project_dir: &ProjectDirs) -> String {
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

fn keycode_restart_default() -> KeyCode {
    KeyCode::Char('r')
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

    #[serde(
        default = "keycode_restart_default",
        with = "KeyCodeDef",
        rename = "continue"
    )]
    pub restart: KeyCode,
}

/// create the help message from a config
impl Config {
    pub fn create_help_message(&self) -> Vec<String> {
        vec![
            "".into(),
            format!("Left: {}", self.left),
            format!("Right: {}", self.right),
            format!("Rotate: {}", self.rotate),
            format!("Soft Drop: {}", self.soft_drop),
            format!("Hard Drop: {}", self.hard_drop),
            format!("Pause: {}", self.pause.to_char()),
            format!("Quit: {}", self.quit.to_char()),
            "".into(),
        ]
    }
}

/// mirrors a subset of the KeyCode definition
///
/// to be able to use Deserialize
/// for the crossterm `KeyCode`
/// adding another Key from the `KeyCode`
/// should be as easy as adding it to this enum
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
    /// Enter key.
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

impl KeyWithAlt {
    pub fn new(a: KeyCode, b: KeyCode) -> Self {
        Self { key: a, alt: b }
    }
}

pub trait MatchesAnyKey {
    fn matches(&self, code: &KeyWithAlt) -> bool;
}

impl MatchesAnyKey for KeyCode {
    /// checks if the code (self) is in the KeyWithAlt arg provided
    fn matches(&self, to_match: &KeyWithAlt) -> bool {
        self == &to_match.key || self == &to_match.alt
    }
}

/// custom trait due to orphan rules
pub trait ToChar {
    fn to_char(&self) -> char;
}
impl ToChar for KeyCode {
    fn to_char(&self) -> char {
        match *self {
            KeyCode::Enter => '↵',
            KeyCode::Esc => '␛',
            KeyCode::Left => '←',
            KeyCode::Right => '→',
            KeyCode::Up => '↑',
            KeyCode::Down => '↓',
            KeyCode::Char(' ') => '·',
            KeyCode::Char(ch) => ch.to_ascii_uppercase(),
            _ => '�',
        }
    }
}

impl std::fmt::Display for KeyWithAlt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key.to_char())?;

        if self.alt != KeyCode::Null {
            write!(f, ", {}", self.alt.to_char())?;
        }

        Ok(())
    }
}

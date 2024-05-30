//! Application config functions
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::path;

include!(concat!(env!("OUT_DIR"), "/config/default.rs"));

/// Toml data struct
#[derive(Deserialize)]
struct ConfigData {
    pub table: Option<TableData>,
}

/// Table config data
#[derive(Deserialize)]
struct TableData {
    pub horizontal: Option<char>,
    pub vertical: Option<char>,
    pub up_horizontal: Option<char>,
    pub down_horizontal: Option<char>,
    pub vertical_right: Option<char>,
    pub vertical_left: Option<char>,
    pub vertical_horizontal: Option<char>,
    pub down_right: Option<char>,
    pub down_left: Option<char>,
    pub up_right: Option<char>,
    pub up_left: Option<char>,
}

/// Application config
#[derive(Debug)]
pub struct Config {
    pub table: TableConfig,
}

impl From<ConfigData> for Config {
    fn from(value: ConfigData) -> Self {
        let table = match value.table {
            // If table config data is Some, use values
            Some(table_data) => TableConfig::new(
                TableHorizontalChars {
                    horizontal: table_data.horizontal.unwrap_or('─'),
                    up_horizontal: table_data.up_horizontal.unwrap_or('┴'),
                    down_horizontal: table_data.down_horizontal.unwrap_or('┬'),
                },
                TableVerticalChars {
                    vertical: table_data.vertical.unwrap_or('│'),
                    vertical_right: table_data.vertical_right.unwrap_or('├'),
                    vertical_left: table_data.vertical_left.unwrap_or('┤'),
                },
                TableCornerChars {
                    down_right: table_data.down_right.unwrap_or('┌'),
                    down_left: table_data.down_left.unwrap_or('┐'),
                    up_right: table_data.up_right.unwrap_or('└'),
                    up_left: table_data.up_left.unwrap_or('┘'),
                },
                table_data.vertical_horizontal.unwrap_or('┼'),
            ),

            // If table config data is none, use defaults
            None => TableConfig::default(),
        };

        Self { table }
    }
}

/// Application Table config
#[derive(Debug)]
pub struct TableConfig {
    pub horizontal: char,
    pub vertical: char,
    pub up_horizontal: char,
    pub down_horizontal: char,
    pub vertical_right: char,
    pub vertical_left: char,
    pub vertical_horizontal: char,
    pub down_right: char,
    pub down_left: char,
    pub up_right: char,
    pub up_left: char,
}

impl TableConfig {
    fn new(
        horizontal_chars: TableHorizontalChars,
        vertical_chars: TableVerticalChars,
        corner_chars: TableCornerChars,
        vertical_horizontal_char: char,
    ) -> Self {
        TableConfig {
            horizontal: horizontal_chars.horizontal,
            vertical: vertical_chars.vertical,
            up_horizontal: horizontal_chars.up_horizontal,
            down_horizontal: horizontal_chars.down_horizontal,
            vertical_right: vertical_chars.vertical_right,
            vertical_left: vertical_chars.vertical_left,
            vertical_horizontal: vertical_horizontal_char,
            down_right: corner_chars.down_right,
            down_left: corner_chars.down_left,
            up_right: corner_chars.up_right,
            up_left: corner_chars.up_left,
        }
    }

    fn default() -> Self {
        TableConfig {
            horizontal: '─',
            up_horizontal: '┴',
            down_horizontal: '┬',
            vertical: '│',
            vertical_right: '├',
            vertical_left: '┤',
            down_right: '┌',
            down_left: '┐',
            up_right: '└',
            up_left: '┘',
            vertical_horizontal: '┼',
        }
    }
}

struct TableHorizontalChars {
    pub horizontal: char,
    pub up_horizontal: char,
    pub down_horizontal: char,
}

struct TableVerticalChars {
    pub vertical: char,
    pub vertical_right: char,
    pub vertical_left: char,
}

struct TableCornerChars {
    pub down_right: char,
    pub down_left: char,
    pub up_right: char,
    pub up_left: char,
}

/// Gets the application config file and returns it as a Config struct. If path is none, gets the
/// config from the default location creating the default file if it doesn't exist
///
/// # Errors
///
/// Will return an error if Some path is not able to be read, or if creation of config file fails
pub fn get_config(path: Option<path::PathBuf>) -> Result<Config, toado::Error> {
    let contents = if let Some(path) = path {
        fs::read_to_string(path)?
    } else {
        let home_dir = env::var("HOME")?;
        let mut path = path::PathBuf::from(format!("{home_dir}/.config/toado/"));

        fs::create_dir_all(path.clone())?;

        path.push("config.toml");

        if path.try_exists().unwrap_or(false) {
            // If config exists in default location, read files
            fs::read_to_string(path)?
        } else {
            // Else write default config to file
            let contents = get_default_config();
            fs::write(path, contents.clone())?;

            // Return default config contents
            contents
        }
    };

    let data: ConfigData = toml::from_str(&contents)?;
    Ok(Config::from(data))
}

//
// private functions
//

/// gets the default contents config.toml as a string
fn get_default_config() -> String {
    default_config()
}

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
    pub list: Option<ListData>,
}

/// Table config data
#[derive(Deserialize)]
struct TableData {
    pub seperate_columns: Option<bool>,
    pub seperate_rows: Option<bool>,
    pub characters: Option<TableCharsData>,
}

/// Table chars config data
#[derive(Deserialize)]
struct TableCharsData {
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

/// List command config data
#[derive(Deserialize)]
struct ListData {
    pub default_verbose: Option<bool>,
}

/// Application config
pub struct Config {
    pub table: TableConfig,
    pub list: ListConfig,
}

impl From<ConfigData> for Config {
    fn from(value: ConfigData) -> Self {
        let mut table = TableConfig::default();

        if let Some(table_data) = value.table {
            if let Some(value) = table_data.seperate_columns {
                table.seperate_cols = value;
            }

            if let Some(value) = table_data.seperate_rows {
                table.seperate_rows = value;
            }

            if let Some(table_chars) = table_data.characters {
                if let Some(value) = table_chars.horizontal {
                    table.horizontal = value
                }

                if let Some(value) = table_chars.up_horizontal {
                    table.up_horizontal = value;
                }

                if let Some(value) = table_chars.down_horizontal {
                    table.down_horizontal = value;
                }

                if let Some(value) = table_chars.vertical {
                    table.vertical = value;
                }

                if let Some(value) = table_chars.vertical_right {
                    table.vertical_right = value;
                }

                if let Some(value) = table_chars.vertical_left {
                    table.vertical_left = value;
                }

                if let Some(value) = table_chars.down_right {
                    table.down_right = value;
                }

                if let Some(value) = table_chars.down_left {
                    table.down_left = value;
                }

                if let Some(value) = table_chars.up_right {
                    table.up_right = value;
                }

                if let Some(value) = table_chars.up_left {
                    table.up_left = value;
                }

                if let Some(value) = table_chars.vertical_horizontal {
                    table.vertical_horizontal = value;
                }
            }
        }

        let mut list = ListConfig::default();

        if let Some(list_data) = value.list {
            if let Some(value) = list_data.default_verbose {
                list.default_verbose = value;
            }
        }

        Self { table, list }
    }
}

/// Application Table config
pub struct TableConfig {
    pub seperate_cols: bool,
    pub seperate_rows: bool,
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
    /// Create a default table config struct
    fn default() -> Self {
        TableConfig {
            seperate_cols: true,
            seperate_rows: false,

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

/// List command config
#[derive(Deserialize)]
pub struct ListConfig {
    pub default_verbose: bool,
}

impl ListConfig {
    pub fn default() -> Self {
        Self {
            default_verbose: false,
        }
    }
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

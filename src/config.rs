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
    pub seperate_columns: Option<bool>,
    pub seperate_rows: Option<bool>,
    pub characters: Option<TableCharsData>,
}

/// Table chars datas
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

/// Application config
#[derive(Debug)]
pub struct Config {
    pub table: TableConfig,
}

impl From<ConfigData> for Config {
    fn from(value: ConfigData) -> Self {
        let mut table = TableConfig::default();

        if let Some(table_data) = value.table {
            table.seperate_cols = table_data.seperate_columns.unwrap_or(true);
            table.seperate_rows = table_data.seperate_rows.unwrap_or(false);

            if let Some(table_chars) = table_data.characters {
                table.horizontal = table_chars.horizontal.unwrap_or('─');
                table.up_horizontal = table_chars.up_horizontal.unwrap_or('┴');
                table.down_horizontal = table_chars.down_horizontal.unwrap_or('┬');
                table.vertical = table_chars.vertical.unwrap_or('│');
                table.vertical_right = table_chars.vertical_right.unwrap_or('├');
                table.vertical_left = table_chars.vertical_left.unwrap_or('┤');
                table.down_right = table_chars.down_right.unwrap_or('┌');
                table.down_left = table_chars.down_left.unwrap_or('┐');
                table.up_right = table_chars.up_right.unwrap_or('└');
                table.up_left = table_chars.up_left.unwrap_or('┘');
                table.vertical_horizontal = table_chars.vertical_horizontal.unwrap_or('┼');
            }
        }

        Self { table }
    }
}

/// Application Table config
#[derive(Debug)]
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

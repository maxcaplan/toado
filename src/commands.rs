//! Toado application commands
use crate::{flags, formatting};

pub use projects::*;
use regex::Regex;
pub use tasks::*;

mod projects;
mod tasks;

//
// Private methods
//

/// Get the input theme used for user input
fn get_input_theme() -> impl dialoguer::theme::Theme {
    dialoguer::theme::ColorfulTheme::default()
}

/// Return the `T` of an `Option<T>` if `Option<T>` is `Some<T>`, otherwise, prompt the user for an
/// input of type `T`.
///
/// # Errors
///
/// Returns error if getting user input fails
fn option_or_input<T>(value: Option<T>, input: dialoguer::Input<T>) -> Result<T, toado::Error>
where
    T: Clone + ToString + std::str::FromStr,
    <T as std::str::FromStr>::Err: ToString,
{
    match value {
        Some(value) => Ok(value),
        None => Ok(input.interact_text()?),
    }
}

/// Return the `Some(T)` of an `Option<T>` if `Option<T>` is `Some(T)`, otherwise,
/// prompt the user for an input of type `T`. If user input is blank, return `None`
///
/// # Errors
///
/// Returns error if getting user input fails
fn option_or_input_option<T>(
    value: Option<T>,
    input: dialoguer::Input<T>,
) -> Result<Option<T>, toado::Error>
where
    T: Clone + ToString + std::str::FromStr,
    <T as std::str::FromStr>::Err: ToString,
{
    match value {
        Some(value) => Ok(Some(value)),
        None => {
            let user_input = input.allow_empty(true).interact_text()?;

            Ok(if !user_input.to_string().is_empty() {
                Some(user_input)
            } else {
                None
            })
        }
    }
}

/// Validate an item name
fn validate_name(input: &str) -> Result<(), String> {
    let r = Regex::new(r"(^[0-9]+$|^\d)").expect("Regex creation should not fail");
    if r.is_match(input) {
        Err("Name cannot start with or be a number".to_string())
    } else {
        Ok(())
    }
}

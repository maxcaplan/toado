use crate::flags;

pub fn create_task(
    args: flags::AddArgs,
    app: toado::Server,
) -> Result<(i64, String), toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let name = option_or_input(
        args.name,
        dialoguer::Input::with_theme(&theme).with_prompt("Name"),
    )?;

    let priority = option_or_input(
        args.item_priority,
        dialoguer::Input::with_theme(&theme).with_prompt("Priority"),
    )?;

    let start_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.start_time,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Start Time (optional)")
                .allow_empty(true),
        )?
    };

    let end_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.end_time,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("End Time (optional)")
                .allow_empty(true),
        )?
    };

    let repeat = if args.optional {
        None
    } else {
        option_or_input_option(
            args.repeat,
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Repeats (optional)")
                .allow_empty(true),
        )?
    };

    let task_id = app.add_task(toado::AddTaskArgs {
        name: String::from(&name),
        priority,
        status: toado::ItemStatus::Incomplete,
        start_time,
        end_time,
        repeat,
        notes: None,
    })?;

    Ok((task_id, name))
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

/// Return the `Some(String)` of an `Option<String>` if `Option<String>` is `Some(String)`, otherwise,
/// prompt the user for an input of type `String`. If user input is blank, return `None`
/// TODO: Make this function generic. ie. `Result<Option<T>, toado::Error>`
///
/// # Errors
///
/// Returns error if getting user input fails
fn option_or_input_option(
    value: Option<String>,
    input: dialoguer::Input<String>,
) -> Result<Option<String>, toado::Error> {
    match value {
        Some(value) => Ok(Some(value)),
        None => {
            let user_input = input.interact_text()?;

            Ok(if !user_input.is_empty() {
                Some(user_input)
            } else {
                None
            })
        }
    }
}
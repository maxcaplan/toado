use super::*;

/// Creates a new project in a toado application. Populates project data with arguments. Prompts
/// user for any data not provided by arguments.
///
/// # Errors
///
/// Will return an error if user input fails or if project creation fails
pub fn create_project(
    args: flags::AddArgs,
    app: toado::Server,
) -> Result<(i64, String), toado::Error> {
    let theme = get_input_theme();

    // Get user Input

    let name = option_or_input(
        args.name,
        dialoguer::Input::with_theme(&theme)
            .with_prompt("Name")
            .validate_with(|input: &String| validate_name(input)),
    )?;

    let start_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.start_time,
            dialoguer::Input::with_theme(&theme).with_prompt("Start Time (optional)"),
        )?
    };

    let end_time = if args.optional {
        None
    } else {
        option_or_input_option(
            args.end_time,
            dialoguer::Input::with_theme(&theme).with_prompt("End Time (optional)"),
        )?
    };

    let notes = if args.optional {
        None
    } else {
        option_or_input_option(
            args.notes,
            dialoguer::Input::with_theme(&theme).with_prompt("Notes (optional)"),
        )?
    };

    // Add project to app database
    let id = app.add_project(toado::AddProjectArgs {
        name: name.clone(),
        start_time,
        end_time,
        notes,
    })?;

    Ok((id, name))
}

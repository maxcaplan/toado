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

/// Get a list of projects from a toado app server
///
/// # Errors
///
/// Will return an error if selecting projects from app database fails, or if getting row count of
/// table in app database fails
pub fn list_projects(
    args: flags::ListArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let (cols, order_by, order_dir, limit, offset) = parse_list_args(&args);

    let projects = app.select_project(cols, None, order_by, order_dir, limit, offset)?;
    let num_projects = projects.len();

    let mut table_string = formatting::format_project_list(projects, true, false, args.verbose);

    // If not selecting all projects, display number of tasks selected
    if !args.full {
        table_string.push_str(&list_footer(
            offset,
            num_projects,
            app.get_table_row_count(toado::Tables::Projects)?,
        ));
    }

    Ok(Some(table_string))
}

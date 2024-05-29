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

/// Updates a project in a toado application. Either updates the project with cli argument values
/// if suplied, or prompts the user for update values
///
/// # Errors
///
/// Will return an error if user input fails, or if project updating fails
pub fn update_project(args: flags::UpdateArgs, app: toado::Server) -> Result<u64, toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let search_term = option_or_input(
        args.term.clone(),
        dialoguer::Input::with_theme(&theme).with_prompt("Project name"),
    )?;

    let project = prompt_project_selection(
        &app,
        search_term,
        toado::QueryCols::Some(vec!["id", "name", "start_time", "end_time"]),
        &theme,
    )?;

    // Get selected project id
    let project_id = match project.id {
        Some(id) => id,
        None => return Err(Into::into("project id should exist")),
    };

    let condition = toado::QueryConditions::Equal {
        col: "id",
        value: project_id,
    }
    .to_string();

    let (name, start_time, end_time, notes) = if args.has_project_update_values() {
        // If update values are set by command arguments, use those values
        (
            toado::UpdateAction::from(args.name),
            nullable_into_update_action(args.start_time),
            nullable_into_update_action(args.end_time),
            nullable_into_update_action(args.notes),
        )
    } else {
        // Else, prompt user for update values

        // Get current project values
        let current_name = match project.name {
            Some(value) => value,
            None => return Err(Into::into("project name should exist")),
        };
        let current_start_time = project.start_time.unwrap_or("".to_string());
        let current_end_time = project.end_time.unwrap_or("".to_string());
        let current_notes = project.notes.unwrap_or("".to_string());

        // Get user input for update values
        let name: String = dialoguer::Input::with_theme(&theme)
            .with_prompt("Name")
            .validate_with(|input: &String| validate_name(input))
            .with_initial_text(current_name)
            .interact_text()?;

        let start_time: String = dialoguer::Input::with_theme(&theme)
            .with_prompt("Start Time (optional)")
            .with_initial_text(current_start_time)
            .allow_empty(true)
            .interact_text()?;

        let end_time: String = dialoguer::Input::with_theme(&theme)
            .with_prompt("End Time (optional)")
            .with_initial_text(current_end_time)
            .allow_empty(true)
            .interact_text()?;

        let notes: String = dialoguer::Input::with_theme(&theme)
            .with_prompt("Notes (optional)")
            .with_initial_text(current_notes)
            .allow_empty(true)
            .interact_text()?;

        (
            toado::UpdateAction::Some(name),
            toado::UpdateAction::from(start_time),
            toado::UpdateAction::from(end_time),
            toado::UpdateAction::from(notes),
        )
    };

    app.update_project(Some(condition), name, start_time, end_time, notes)
}

pub fn delete_project(
    args: flags::DeleteArgs,
    app: toado::Server,
) -> Result<Option<i64>, toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let search_term = option_or_input(
        args.term,
        dialoguer::Input::with_theme(&theme).with_prompt("Project name"),
    )?;

    let project = prompt_project_selection(
        &app,
        search_term,
        toado::QueryCols::Some(vec!["id", "name", "start_time"]),
        &theme,
    )?;

    // Get selected task id
    let id = match project.id {
        Some(id) => id,
        None => return Err(Into::into("project id should exist")),
    };

    let affected_rows = app.delete_project(Some(
        toado::QueryConditions::Equal {
            col: "id",
            value: id,
        }
        .to_string(),
    ))?;

    if affected_rows >= 1 {
        Ok(Some(id))
    } else {
        Err(Into::into("no project deleted"))
    }
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

//
// Private Methods
//

/// Selects projects from an application database given a search term. If multiple projects match the
/// term, prompts the user to select one of the matching projects and returns it. If one project matches
/// inputed name, returns said project
///
/// # Errors
///
/// Will return an error if no projects match the search term
fn prompt_project_selection(
    app: &toado::Server,
    search_term: String,
    cols: toado::QueryCols,
    theme: &dyn dialoguer::theme::Theme,
) -> Result<toado::Project, toado::Error> {
    let select_condition = match search_term.parse::<usize>() {
        // If search term is number, select by id
        Ok(num) => toado::QueryConditions::Equal {
            col: "id",
            value: num.to_string(),
        },
        // If search term is not number, select by name
        Err(_) => toado::QueryConditions::Like {
            col: "name",
            value: format!("'%{search_term}%'"),
        },
    };

    // Get tasks matching name argument
    let mut projects = app.select_project(
        // toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
        cols,
        Some(select_condition.to_string()),
        Some(toado::OrderBy::Name),
        None,
        Some(toado::RowLimit::All),
        None,
    )?;

    // If no tasks match search term, return error
    if projects.is_empty() {
        return Err(Into::into(format!("no project matches {search_term}")));
    }

    if projects.len() == 1 {
        Ok(projects.remove(0))
    }
    // If multiple tasks match name argument, prompt user to select one
    else {
        // Format matching tasks into vector of strings
        let project_strings: Vec<String> =
            formatting::format_project_list(projects.clone(), true, false, false)
                .split('\n')
                .map(|line| line.to_string())
                .collect();

        // Get task selection from user
        match projects.get(
            dialoguer::Select::with_theme(theme)
                .with_prompt("Select project")
                .items(&project_strings)
                .interact()?,
        ) {
            Some(project) => Ok(project.clone()),
            None => Err(Into::into("selected project should exist")),
        }
    }
}

use super::*;

/// Creates a new task in a toado server with provided arguments. Prompts the user to input any task
/// information not provided in the arguments.
///
/// # Errors
/// Will return an error if any of the user input prompts fail, or if the creation of the task
/// fails.
pub fn create_task(
    args: flags::AddArgs,
    app: toado::Server,
) -> Result<(i64, String), toado::Error> {
    let theme = get_input_theme();

    let name = option_or_input(
        args.name,
        dialoguer::Input::with_theme(&theme)
            .with_prompt("Name")
            .validate_with(|input: &String| validate_name(input)),
    )?;

    let priority = option_or_input(
        args.item_priority,
        dialoguer::Input::with_theme(&theme)
            .with_prompt("Priority")
            .default(0),
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

    let repeat = if args.optional {
        None
    } else {
        option_or_input_option(
            args.repeat,
            dialoguer::Input::with_theme(&theme).with_prompt("Repeats (optional)"),
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

    let task_id = app.add_task(toado::AddTaskArgs {
        name: String::from(&name),
        priority,
        status: toado::ItemStatus::Incomplete,
        start_time,
        end_time,
        repeat,
        notes,
    })?;

    Ok((task_id, name))
}

/// Deletes a task in a toado server database. Searches for task to delete with given search term,
/// or prompts user for search term if one is not provided
///
/// # Errors
///
/// Will return an error if user input fails, if deletion operation fails, or if no tasks are
/// deleted
pub fn delete_task(
    args: flags::DeleteArgs,
    app: toado::Server,
) -> Result<Option<i64>, toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let search_term = option_or_input(
        args.term,
        dialoguer::Input::with_theme(&theme).with_prompt("Task name"),
    )?;

    let task = prompt_task_selection(
        &app,
        search_term,
        toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
        &theme,
    )?;

    // Get selected task id
    let id = match task.id {
        Some(id) => id,
        None => return Err(Into::into("task id should exist")),
    };

    let affected_rows = app.delete_task(Some(
        toado::QueryConditions::Equal {
            col: "id",
            value: id,
        }
        .to_string(),
    ))?;

    if affected_rows >= 1 {
        Ok(Some(id))
    } else {
        Err(Into::into("no tasks deleted"))
    }
}

/// Update a task in a toado server
///
/// # Errors
///
/// Will return an error if user input fails, if task updating fails, or if no task is updated
pub fn update_task(args: flags::UpdateArgs, app: toado::Server) -> Result<u64, toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let search_term = option_or_input(
        args.term.clone(),
        dialoguer::Input::with_theme(&theme).with_prompt("Task name"),
    )?;

    let task = prompt_task_selection(
        &app,
        search_term,
        toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
        &theme,
    )?;

    // Get selected task id
    let task_id = match task.id {
        Some(id) => id,
        None => return Err(Into::into("task id should exist")),
    };

    let (name, priority, start_time, end_time, repeat, notes) = {
        if args.has_task_update_values() {
            // If update values are set by command arguments, use those values
            (
                toado::UpdateAction::from(args.name),
                toado::UpdateAction::from(args.item_priority),
                nullable_into_update_action(args.start_time),
                nullable_into_update_action(args.end_time),
                nullable_into_update_action(args.repeat),
                nullable_into_update_action(args.notes),
            )
        } else {
            // Else, prompt user for update values

            // Get current task values
            let current_name = match task.name {
                Some(value) => value,
                None => return Err(Into::into("task name should exist")),
            };
            let current_priority = match task.priority {
                Some(value) => value,
                None => return Err(Into::into("task priority should exist")),
            };
            let current_start_time = task.start_time.unwrap_or("".to_string());
            let current_end_time = task.end_time.unwrap_or("".to_string());
            let current_repeat = task.repeat.unwrap_or("".to_string());
            let current_notes = task.notes.unwrap_or("".to_string());

            // Get user input for update values
            let name: String = dialoguer::Input::with_theme(&theme)
                .with_prompt("Name")
                .validate_with(|input: &String| validate_name(input))
                .with_initial_text(current_name)
                .interact_text()?;

            let priority: u64 = dialoguer::Input::with_theme(&theme)
                .with_prompt("Priority")
                .default(0)
                .with_initial_text(current_priority.to_string())
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

            let repeat: String = dialoguer::Input::with_theme(&theme)
                .with_prompt("Repeat (optional)")
                .with_initial_text(current_repeat)
                .allow_empty(true)
                .interact_text()?;

            let notes: String = dialoguer::Input::with_theme(&theme)
                .with_prompt("Notes (optional)")
                .with_initial_text(current_notes)
                .allow_empty(true)
                .interact_text()?;

            fn string_to_update_action(s: String) -> toado::UpdateAction<String> {
                if s.is_empty() {
                    toado::UpdateAction::Null
                } else {
                    toado::UpdateAction::Some(format!("'{s}'"))
                }
            }

            (
                toado::UpdateAction::Some(name),
                toado::UpdateAction::Some(priority),
                string_to_update_action(start_time),
                string_to_update_action(end_time),
                string_to_update_action(repeat),
                string_to_update_action(notes),
            )
        }
    };

    app.update_task(
        Some(
            toado::QueryConditions::Equal {
                col: "id",
                value: task_id,
            }
            .to_string(),
        ),
        toado::UpdateTaskArgs {
            name,
            priority,
            status: toado::UpdateAction::None,
            start_time,
            end_time,
            repeat,
            notes,
        },
    )
}

/// Searches for a task in a toado server database with provided search term. If term is a positive
/// integer, searches by task id, otherwise searches by name
///
/// # Errors
///
/// Will return an error if task selection fails
pub fn search_tasks(
    args: flags::SearchArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let condition = match args.term.parse::<usize>() {
        // If search term is number, select by id
        Ok(value) => toado::QueryConditions::Equal {
            col: "id",
            value: value.to_string(),
        },
        // If search term is not number, select by name
        Err(_) => toado::QueryConditions::Like {
            col: "name",
            value: format!("'%{}%'", args.term),
        },
    };

    let tasks = app.select_tasks(
        toado::QueryCols::All,
        Some(condition.to_string()),
        Some(toado::OrderBy::Id),
        None,
        Some(toado::RowLimit::All),
        None,
    )?;

    if tasks.is_empty() {
        Ok(None)
    } else if tasks.len() == 1 {
        Ok(Some(formatting::format_task(tasks[0].clone())))
    } else {
        Ok(Some(formatting::format_task_list(
            tasks,
            true,
            false,
            args.verbose,
        )))
    }
}

/// Gets a list of tasks from a toado server
///
/// # Errors
///
/// Will return an error if selecting tasks from the server database fails
pub fn list_tasks(
    args: flags::ListArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let (cols, order_by, order_dir, limit, offset) = parse_list_args(&args);

    // Get tasks from application database
    let tasks = app.select_tasks(cols, None, order_by, order_dir, limit, offset)?;
    let num_tasks = tasks.len();

    // Format tasks into a table string to display
    let mut table_string = formatting::format_task_list(tasks, true, false, args.verbose);

    // If not selecting all tasks, display number of tasks selected
    if !args.full {
        table_string.push_str(&list_footer(
            offset,
            num_tasks,
            app.get_table_row_count(toado::Tables::Tasks)?,
        ));
    }

    Ok(Some(table_string))
}

pub fn check_task(
    args: flags::CheckArgs,
    app: toado::Server,
) -> Result<(String, toado::ItemStatus), toado::Error> {
    let theme = dialoguer::theme::ColorfulTheme::default();

    let search_term = option_or_input(
        args.term,
        dialoguer::Input::with_theme(&theme).with_prompt("Task name"),
    )?;

    let task = prompt_task_selection(
        &app,
        search_term,
        toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
        &theme,
    )?;

    // Get selected task id
    let id = match task.id {
        Some(id) => id,
        None => return Err(Into::into("task id should exist")),
    };

    let name = match task.name {
        Some(name) => name,
        None => return Err(Into::into("task name should exist")),
    };

    let new_status = match args.incomplete {
        true => toado::ItemStatus::Incomplete,
        false => toado::ItemStatus::Complete,
    };

    let affected_rows = app.update_task(
        Some(
            toado::QueryConditions::Equal {
                col: "id",
                value: id,
            }
            .to_string(),
        ),
        toado::UpdateTaskArgs::update_status(new_status),
    )?;

    if affected_rows == 0 {
        Err(Into::into("no rows affected by update"))
    } else {
        Ok((name, new_status))
    }
}

//
// Private Methods
//

/// Selects tasks from an application database given a search term. If multiple tasks match the
/// term, prompts the user to select one of the matching tasks and returns it. If one task matches
/// inputed name, returns said task
///
/// # Errors
/// Will return an error if no tasks match the search term
fn prompt_task_selection(
    app: &toado::Server,
    search_term: String,
    cols: toado::QueryCols,
    theme: &dyn dialoguer::theme::Theme,
) -> Result<toado::Task, toado::Error> {
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
    let mut tasks = app.select_tasks(
        // toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
        cols,
        Some(select_condition.to_string()),
        Some(toado::OrderBy::Name),
        None,
        Some(toado::RowLimit::All),
        None,
    )?;

    // If no tasks match search term, return error
    if tasks.is_empty() {
        return Err(Into::into(format!("no task matches {search_term}")));
    }

    if tasks.len() == 1 {
        Ok(tasks.remove(0))
    }
    // If multiple tasks match name argument, prompt user to select one
    else {
        // Format matching tasks into vector of strings
        let task_strings: Vec<String> =
            formatting::format_task_list(tasks.clone(), true, false, false)
                .split('\n')
                .map(|line| line.to_string())
                .collect();

        // Get task selection from user
        match tasks.get(
            dialoguer::Select::with_theme(theme)
                .with_prompt("Select task")
                .items(&task_strings)
                .interact()?,
        ) {
            Some(task) => Ok(task.clone()),
            None => Err(Into::into("selected task should exist")),
        }
    }
}

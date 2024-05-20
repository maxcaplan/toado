//! Toado application commands
use crate::{flags, formatting};
use regex::Regex;

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
    let theme = dialoguer::theme::ColorfulTheme::default();

    let name = option_or_input(
        args.name,
        dialoguer::Input::with_theme(&theme)
            .with_prompt("Name")
            .validate_with(|input: &String| {
                let r = Regex::new(r"(^[0-9]+$|^\d)").expect("Regex creation should not fail");
                if r.is_match(input) {
                    Err("Name cannot start or be a number")
                } else {
                    Ok(())
                }
            }),
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

//
// Private methods
//

/// Gets a list of tasks from a toado server
///
/// # Errors
///
/// Will return an error if selecting tasks from the server database fails
pub fn list_tasks(
    args: &flags::ListArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    // Determin order direction
    let order_dir = match (args.asc, args.desc) {
        (true, _) => Some(toado::OrderDir::Asc),
        (false, true) => Some(toado::OrderDir::Desc),
        (false, false) => None,
    };

    // Determin columns to select
    let cols = if args.verbose {
        toado::QueryCols::All
    } else {
        toado::QueryCols::Some(Vec::from(["id", "name", "priority", "status"]))
    };

    // Determin selection row limit
    let limit = match (args.full, args.limit) {
        (true, _) => Some(toado::RowLimit::All), // Select all
        (false, Some(val)) => Some(toado::RowLimit::Limit(val)), // Select set number
        _ => None,                               // Select default number
    };

    // Get tasks from application database
    let tasks = app.select_tasks(cols, None, args.order_by, order_dir, limit, args.offset)?;
    let num_tasks = tasks.len();

    // Format tasks into a table string to display
    let mut table_string = formatting::format_task_list(tasks, true, false, args.verbose);

    // If not selecting all tasks, display number of tasks selected
    if !args.full {
        let start_pos = args.offset.unwrap_or(0);
        table_string.push_str(&format!(
            "\n{}-{} of {}",
            start_pos,
            start_pos + num_tasks,
            app.get_table_row_count(toado::Tables::Tasks)?,
        ))
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
        toado::UpdateTaskCols::status(new_status),
        Some(
            toado::QueryConditions::Equal {
                col: "id",
                value: id,
            }
            .to_string(),
        ),
    )?;

    if affected_rows == 0 {
        Err(Into::into("no rows affected by update"))
    } else {
        Ok((name, new_status))
    }
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

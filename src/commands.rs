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

    let tasks = app.select_tasks_condition(
        toado::SelectCols::All,
        vec![(condition, None)],
        Some(toado::OrderBy::Id),
        None,
        Some(toado::SelectLimit::All),
        None,
    )?;

    if tasks.is_empty() {
        Ok(None)
    } else if tasks.len() == 1 {
        Ok(Some(formatting::format_task(tasks[0].clone())))
    } else {
        Ok(Some(formatting::format_task_list(tasks, args.verbose)))
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
        toado::SelectCols::All
    } else {
        toado::SelectCols::Some(Vec::from(["id", "name", "priority", "status"]))
    };

    // Determin selection row limit
    let limit = match (args.full, args.limit) {
        (true, _) => Some(toado::SelectLimit::All), // Select all
        (false, Some(val)) => Some(toado::SelectLimit::Limit(val)), // Select set number
        _ => None,                                  // Select default number
    };

    // Get tasks from application database
    let tasks = app.select_tasks(cols, args.order_by, order_dir, limit, args.offset)?;
    let num_tasks = tasks.len();

    // Format tasks into a table string to display
    let mut table_string = formatting::format_task_list(tasks, args.verbose);

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

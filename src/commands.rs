//! Toado application commands
use crate::{
    flags,
    formatting::{self},
};

pub use assignment::*;
pub use projects::*;
pub use tasks::*;

use regex::Regex;

mod assignment;
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

enum TasksOrProjects {
    Tasks(Vec<toado::Task>),
    Projects(Vec<toado::Project>),
}

impl TasksOrProjects {
    fn tasks(self) -> Vec<toado::Task> {
        match self {
            Self::Tasks(tasks) => tasks,
            _ => panic!("value should be of type Tasks"),
        }
    }

    fn projects(self) -> Vec<toado::Project> {
        match self {
            Self::Projects(projects) => projects,
            _ => panic!("value should be of type Projects"),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Tasks(tasks) => tasks.is_empty(),
            Self::Projects(projects) => projects.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Tasks(tasks) => tasks.len(),
            Self::Projects(projects) => projects.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Tasks(_) => "tasks",
            Self::Projects(_) => "projects",
        }
    }
}

/// Prompt the user to select an item (Task or Project) from list of items from a toado
/// application. If search term is Some, filters list to matching items.
///
/// # Errors
///
/// Will return an error if getting item list fails, or if user input fails
fn prompt_select_item(
    term: Option<String>,
    app: &toado::Server,
    theme: &dyn dialoguer::theme::Theme,
    multi_select: bool,
    projects: bool,
) -> Result<TasksOrProjects, toado::Error> {
    let condition = match &term {
        Some(term) => match term.parse::<usize>() {
            Ok(num) => Some(
                toado::QueryConditions::Equal {
                    col: "id",
                    value: num.to_string(),
                }
                .to_string(),
            ),
            Err(_) => Some(
                toado::QueryConditions::Like {
                    col: "name",
                    value: format!("'%{term}%'"),
                }
                .to_string(),
            ),
        },
        None => None,
    };

    let items = if !projects {
        TasksOrProjects::Tasks(app.select_tasks(
            toado::QueryCols::Some(vec!["id", "name", "priority", "status"]),
            condition,
            Some(toado::OrderBy::Name),
            None,
            None,
            None,
        )?)
    } else {
        TasksOrProjects::Projects(app.select_project(
            toado::QueryCols::Some(vec!["id", "name", "start_time", "end_time"]),
            condition,
            Some(toado::OrderBy::Name),
            None,
            None,
            None,
        )?)
    };

    if items.is_empty() {
        if let Some(term) = term {
            return Err(Into::into(format!("no {} match {term}", items.name())));
        }

        return Err(Into::into(format!("no {} found", items.name())));
    }

    if items.len() == 1 {
        return Ok(match items {
            TasksOrProjects::Tasks(tasks) => TasksOrProjects::Tasks(vec![tasks[0].clone()]),
            TasksOrProjects::Projects(projects) => {
                TasksOrProjects::Projects(vec![projects[0].clone()])
            }
        });
    }

    let list_string = match &items {
        TasksOrProjects::Tasks(tasks) => {
            formatting::format_task_list(tasks.clone(), true, false, false)
        }
        TasksOrProjects::Projects(projects) => {
            formatting::format_project_list(projects.clone(), true, false, false)
        }
    };

    let select_items: Vec<&str> = list_string.split('\n').collect();

    if multi_select {
        let selected_ids = dialoguer::MultiSelect::with_theme(theme)
            .with_prompt(format!("Select {}", items.name()))
            .items(&select_items)
            .interact()?;

        Ok(match items {
            TasksOrProjects::Tasks(tasks) => {
                let mut selected_tasks: Vec<toado::Task> = Vec::new();

                for idx in selected_ids {
                    if let Some(task) = tasks.get(idx) {
                        selected_tasks.push(task.clone())
                    } else {
                        return Err(Into::into("selected task should exist"));
                    }
                }

                TasksOrProjects::Tasks(selected_tasks)
            }
            TasksOrProjects::Projects(projects) => {
                let mut selected_projects: Vec<toado::Project> = Vec::new();

                for idx in selected_ids {
                    if let Some(project) = projects.get(idx) {
                        selected_projects.push(project.clone())
                    } else {
                        return Err(Into::into("selected project should exist"));
                    }
                }

                TasksOrProjects::Projects(selected_projects)
            }
        })
    } else {
        let selected_idx = dialoguer::Select::with_theme(theme)
            .with_prompt(format!("Select {}", items.name()))
            .items(&select_items)
            .interact()?;

        match items {
            TasksOrProjects::Tasks(tasks) => match tasks.get(selected_idx) {
                Some(task) => Ok(TasksOrProjects::Tasks(vec![task.clone()])),
                None => Err(Into::into("selected task should exist")),
            },
            TasksOrProjects::Projects(projects) => match projects.get(selected_idx) {
                Some(project) => Ok(TasksOrProjects::Projects(vec![project.clone()])),
                None => Err(Into::into("selected project should exist")),
            },
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

/// Parse list command CLI arguments into their respecitve data types
fn parse_list_args<'a>(
    args: &flags::ListArgs,
) -> (
    toado::QueryCols<'a>,
    Option<toado::OrderBy>,
    Option<toado::OrderDir>,
    Option<toado::RowLimit>,
    Option<usize>,
) {
    let order_dir = match (args.asc, args.desc) {
        (true, _) => Some(toado::OrderDir::Asc),
        (false, true) => Some(toado::OrderDir::Desc),
        (false, false) => None,
    };

    // Determin columns to select
    let cols = if args.verbose {
        toado::QueryCols::All
    } else if args.task || !args.project {
        toado::QueryCols::Some(Vec::from(["id", "name", "priority", "status"]))
    } else {
        toado::QueryCols::Some(Vec::from(["id", "name", "start_time", "end_time"]))
    };

    // Determin selection row limit
    let limit = match (args.full, args.limit) {
        (true, _) => Some(toado::RowLimit::All), // Select all
        (false, Some(val)) => Some(toado::RowLimit::Limit(val)), // Select set number
        _ => None,                               // Select default number
    };

    (cols, args.order_by, order_dir, limit, args.offset)
}

fn list_footer(offset: Option<usize>, count: usize, total: usize) -> String {
    let offset = offset.unwrap_or(0);
    format!("\n{}-{} of {}", offset, offset + count, total)
}

/// Converts an optional nullable string into an update action
fn nullable_into_update_action(flag: Option<flags::NullableString>) -> toado::UpdateAction<String> {
    match flag {
        Some(flags::NullableString::Some(value)) => toado::UpdateAction::Some(value),
        Some(flags::NullableString::Null) => toado::UpdateAction::Null,
        None => toado::UpdateAction::None,
    }
}

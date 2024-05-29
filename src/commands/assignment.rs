use crate::flags;

use super::{get_input_theme, prompt_select_item};

// Assigns a single task to a single project in a toado application. Requires a search term to be
// set for both task and project
//
// # Errors
//
// Will return an error if no search term is supplied for task or project, or if task or project
// selection fails, or if no task or project matches respective search term, or if assignment of
// task fails
pub fn assign_task(
    args: flags::AssignArgs,
    app: toado::Server,
) -> Result<(String, String), toado::Error> {
    let (task_term, project_term) = parse_search_terms(&args);
    let task_term = match task_term {
        Some(term) => term,
        None => return Err(Into::into("task search term should be Some")),
    };
    let project_term = match project_term {
        Some(term) => term,
        None => return Err(Into::into("project search term should be Some")),
    };

    let (task_id, task_name, project_id, project_name) =
        match_single_task_and_project(task_term, project_term, &app)?;

    app.assign_task(task_id, project_id)?;
    Ok((task_name, project_name))
}

// Assigns one or more tasks to one or more projects in a toado app. Will prompt user for task and
// or project selection if no search terms are provided as args, or if search terms don't match a
// single task or project.
//
// # Errors
//
// Will return an error if selection of tasks or projects fails, or if task assignment fails
pub fn assign_multiple_tasks(
    args: flags::AssignArgs,
    app: toado::Server,
) -> Result<Vec<(String, String)>, toado::Error> {
    let (task_term, project_term) = parse_search_terms(&args);

    let theme = get_input_theme();

    // Get task(s) to assign to project(s)
    let tasks = prompt_select_item(task_term, &app, &theme, true, false)?.tasks();
    // Get project(s) to assign to tasks(s)
    let projects = prompt_select_item(project_term, &app, &theme, true, true)?.projects();

    let (task_ids, task_names) = parse_task_names_and_ids(tasks)?;
    let (project_ids, project_names) = parse_project_names_and_ids(projects)?;

    let assignment_ids = create_id_pairs(task_ids, project_ids);

    let assignment_names = create_name_pairs(task_names, project_names);

    // Assign tasks to projects
    app.batch_assign_tasks(assignment_ids)?;
    Ok(assignment_names)
}

// Unassigns a task from a project in a toado app. Requires a search term for both tasks and
// projects
//
// # Errors
//
// Will return an error if unable to match a task or project to respective search term, or if
// unassigning fails
pub fn unassign_task(
    args: flags::AssignArgs,
    app: toado::Server,
) -> Result<(String, String), toado::Error> {
    let (task_term, project_term) = parse_search_terms(&args);
    let task_term = match task_term {
        Some(term) => term,
        None => return Err(Into::into("task search term should be Some")),
    };
    let project_term = match project_term {
        Some(term) => term,
        None => return Err(Into::into("project search term should be Some")),
    };

    let (task_id, task_name, project_id, project_name) =
        match_single_task_and_project(task_term, project_term, &app)?;

    app.unassign_task(task_id, project_id)?;
    Ok((task_name, project_name))
}

pub fn unassign_multiple_tasks(
    args: flags::AssignArgs,
    app: toado::Server,
) -> Result<Vec<(String, String)>, toado::Error> {
    let theme = get_input_theme();

    // Get search term(s) for tasks and project(s)
    let (task_term, project_term) = parse_search_terms(&args);

    // Get task(s) to unassign to project(s)
    let tasks = prompt_select_item(task_term, &app, &theme, true, false)?.tasks();
    // Get project(s) to unassign to tasks(s)
    let projects = prompt_select_item(project_term, &app, &theme, true, true)?.projects();

    let (task_ids, task_names) = parse_task_names_and_ids(tasks)?;
    let (project_ids, project_names) = parse_project_names_and_ids(projects)?;

    let unassignment_ids = create_id_pairs(task_ids, project_ids);
    let unassignment_names = create_name_pairs(task_names, project_names);

    app.batch_unassign_tasks(unassignment_ids)?;
    Ok(unassignment_names)
}

//
// Private Functions
//

/// Parse assign args search terms
fn parse_search_terms(args: &flags::AssignArgs) -> (Option<String>, Option<String>) {
    let task_term = match (&args.task, &args.task_term) {
        (Some(term), _) => Some(term.clone()), // Use positional value if Some
        (None, Some(term)) => Some(term.clone()), // Use flag value if positional is None
        (None, None) => None,
    };

    let project_term = match (&args.project, &args.project_term) {
        (Some(term), _) => Some(term.clone()), // Use positional value if Some
        (None, Some(term)) => Some(term.clone()), // Use flag value if positional is None
        (None, None) => None,
    };

    (task_term, project_term)
}

/// Returns the first task and project that matches respective search term
///
/// # Errors
///
/// Will return an error if selecting tasks or projects fails, or if no task or project matches
/// respective search term
fn match_single_task_and_project(
    task_term: String,
    project_term: String,
    app: &toado::Server,
) -> Result<(i64, String, i64, String), toado::Error> {
    // Select tasks matching search term
    let tasks = app.select_tasks(
        toado::QueryCols::Some(vec!["id", "name"]),
        Some(
            match task_term.parse::<i64>() {
                Ok(num) => toado::QueryConditions::Equal {
                    col: "id",
                    value: num.to_string(),
                },
                Err(_) => toado::QueryConditions::Like {
                    col: "name",
                    value: format!("'%{task_term}%'"),
                },
            }
            .to_string(),
        ),
        Some(toado::OrderBy::Name),
        None,
        None,
        None,
    )?;

    if tasks.is_empty() {
        return Err(Into::into(format!("no tasks match '{task_term}'")));
    }

    // Select tasks matching search term
    let projects = app.select_project(
        toado::QueryCols::Some(vec!["id", "name"]),
        Some(
            match project_term.parse::<i64>() {
                Ok(num) => toado::QueryConditions::Equal {
                    col: "id",
                    value: num.to_string(),
                },
                Err(_) => toado::QueryConditions::Like {
                    col: "name",
                    value: format!("'%{project_term}%'"),
                },
            }
            .to_string(),
        ),
        Some(toado::OrderBy::Name),
        None,
        None,
        None,
    )?;

    if projects.is_empty() {
        return Err(Into::into(format!("no project match '{project_term}'")));
    }

    let task = &tasks[0];
    let project = &projects[0];

    let (task_id, task_name) = (
        match &task.id {
            Some(id) => *id,
            None => return Err(Into::into("task should have id")),
        },
        match &task.name {
            Some(name) => name.clone(),
            None => return Err(Into::into("task should have name")),
        },
    );

    let (project_id, project_name) = (
        match &project.id {
            Some(id) => *id,
            None => return Err(Into::into("project should have id")),
        },
        match &project.name {
            Some(name) => name.clone(),
            None => return Err(Into::into("project should have name")),
        },
    );

    Ok((task_id, task_name, project_id, project_name))
}

/// Get task id(s) and name(s)
///
/// # Errors
///
/// Will return an error if task name or id is None
fn parse_task_names_and_ids(
    tasks: Vec<toado::Task>,
) -> Result<(Vec<i64>, Vec<String>), toado::Error> {
    Ok(tasks
        .into_iter()
        .map(|task| match (task.id, task.name) {
            (Some(id), Some(name)) => Ok((id, name)),
            (None, _) => Err("Error: couldn't get task id"),
            (_, None) => Err("Error: couldn't get task name"),
        })
        .collect::<Result<Vec<(i64, String)>, &str>>()?
        .into_iter()
        .unzip())
}

/// Get project id(s) and name(s)
///
/// # Errors
///
/// Will return an error if project name or id is None
fn parse_project_names_and_ids(
    projects: Vec<toado::Project>,
) -> Result<(Vec<i64>, Vec<String>), toado::Error> {
    Ok(projects
        .into_iter()
        .map(|project| match (project.id, project.name) {
            (Some(id), Some(name)) => Ok((id, name)),
            (None, _) => Err("Error: couldn't get project id"),
            (_, None) => Err("Error: couldn't get project name"),
        })
        .collect::<Result<Vec<(i64, String)>, &str>>()?
        .into_iter()
        .unzip())
}

/// Create pairs of task ids with project ids
fn create_id_pairs(task_ids: Vec<i64>, project_ids: Vec<i64>) -> Vec<(i64, i64)> {
    task_ids
        .into_iter()
        .flat_map(|task_id| {
            project_ids
                .clone()
                .into_iter()
                .map(|project_id| (task_id, project_id))
                .collect::<Vec<(i64, i64)>>()
        })
        .collect::<Vec<(i64, i64)>>()
}

/// Create pairs of task names with project names
fn create_name_pairs(task_names: Vec<String>, project_names: Vec<String>) -> Vec<(String, String)> {
    task_names
        .into_iter()
        .flat_map(|task_name| {
            project_names
                .clone()
                .into_iter()
                .map(|project_name| (task_name.clone(), project_name))
                .collect::<Vec<(String, String)>>()
        })
        .collect::<Vec<(String, String)>>()
}

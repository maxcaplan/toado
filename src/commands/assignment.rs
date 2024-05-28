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
    let task_term = match (args.task, args.task_term) {
        (Some(term), _) => term,    // Use positional value if Some
        (None, Some(term)) => term, // Use flag value if positional is None and flag is Some
        (None, None) => return Err(Into::into("task search term should be Some")),
    };

    let project_term = match (args.project, args.project_term) {
        (Some(term), _) => term,    // Use positional value if Some
        (None, Some(term)) => term, // Use flag value if positional is None and flag is Some
        (None, None) => return Err(Into::into("project search term should be Some")),
    };

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
    let task_term = match (args.task, args.task_term) {
        (Some(term), _) => Some(term),    // Use positional value if Some
        (None, Some(term)) => Some(term), // Use flag value if positional is None and flag is Some
        (None, None) => None,
    };

    let project_term = match (args.project, args.project_term) {
        (Some(term), _) => Some(term),    // Use positional value if Some
        (None, Some(term)) => Some(term), // Use flag value if positional is None and flag is Some
        (None, None) => None,
    };

    let theme = get_input_theme();

    // Get task(s) to assign to project(s)
    let tasks = prompt_select_item(task_term, &app, &theme, true, false)?.tasks();
    // Get project(s) to assign to tasks(s)
    let projects = prompt_select_item(project_term, &app, &theme, true, true)?.projects();

    // Get task id(s) and name(s)
    let (task_ids, task_names): (Vec<i64>, Vec<String>) = tasks
        .into_iter()
        .map(|task| match (task.id, task.name) {
            (Some(id), Some(name)) => Ok((id, name)),
            (None, _) => Err("Error: couldn't get task id"),
            (_, None) => Err("Error: couldn't get task name"),
        })
        .collect::<Result<Vec<(i64, String)>, &str>>()?
        .into_iter()
        .unzip();

    // Get project id(s) and name(s)
    let (project_ids, project_names): (Vec<i64>, Vec<String>) = projects
        .into_iter()
        .map(|project| match (project.id, project.name) {
            (Some(id), Some(name)) => Ok((id, name)),
            (None, _) => Err("Error: couldn't get project id"),
            (_, None) => Err("Error: couldn't get project name"),
        })
        .collect::<Result<Vec<(i64, String)>, &str>>()?
        .into_iter()
        .unzip();

    // Create pairs of task ids with project ids
    let assignment_ids: Vec<(i64, i64)> = task_ids
        .into_iter()
        .flat_map(|task_id| {
            project_ids
                .clone()
                .into_iter()
                .map(|project_id| (task_id, project_id))
                .collect::<Vec<(i64, i64)>>()
        })
        .collect();

    // Create pairs of task names with project names
    let assignment_names: Vec<(String, String)> = task_names
        .into_iter()
        .flat_map(|task_name| {
            project_names
                .clone()
                .into_iter()
                .map(|project_name| (task_name.clone(), project_name))
                .collect::<Vec<(String, String)>>()
        })
        .collect();

    // Assign tasks to projects
    app.batch_assign_tasks(assignment_ids)?;
    Ok(assignment_names)
}

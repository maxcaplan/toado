use crate::{config, formatting::table::AsciiTable};

/// Format a single task as a string to be displayed to the user
pub fn format_task(task: toado::Task, config: &config::Config) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Push task id and or name
    if let Some(name) = task.name {
        let name_l = name.len();

        if let Some(id) = task.id {
            let id = id.to_string();
            let id_l = id.len();

            lines.push(format!("{} {} {}", name, config.table.vertical, id));
            lines.push(format!(
                "{}{}{}",
                config.table.horizontal.to_string().repeat(name_l + 1),
                config.table.up_horizontal,
                config.table.horizontal.to_string().repeat(id_l + 1)
            ))
        } else {
            lines.push(name);
            lines.push(config.table.horizontal.to_string().repeat(name_l))
        }
    }

    // Push task priority
    if let Some(priority) = task.priority {
        lines.push(format!("Priority: {priority}"));
    }

    // Push status
    if let Some(status) = task.status {
        lines.push(format!("Status: {}", status.to_string().to_uppercase()));
    }

    // Push task start and or end time
    if let Some(start_time) = task.start_time {
        lines.push(format!("Start: {start_time}"));
        if let Some(end_time) = task.end_time {
            lines.push(format!(" End: {end_time}"));
        }
    } else if let Some(end_time) = task.end_time {
        lines.push(format!("End: {end_time}"));
    }

    // Push repeat
    if let Some(repeat) = task.repeat {
        lines.push(format!("Repeats: {repeat}"));
    }

    // Push projects
    // if let Some(projects) = task.projects {
    //     lines.push(format!("Projects: {}", projects.join(", ")));
    // }

    // Push notes
    if let Some(notes) = task.notes {
        lines.push(format!("Notes: {notes}"))
    }

    lines.join("\n")
}

/// Format a vector of tasks as a string to be displayed to the user
pub fn format_task_list(
    tasks: Vec<toado::Task>,
    seperate_cols: bool,
    seperate_rows: bool,
    verbose: bool,
    config: &config::TableConfig,
) -> String {
    let table = AsciiTable::new(
        tasks
            .into_iter()
            .map(|task| {
                let mut cols = vec![
                    task.id.map_or_else(|| "-".to_string(), |v| v.to_string()),
                    task.name.unwrap_or("-".to_string()),
                    task.priority
                        .map_or_else(|| "-".to_string(), |v| v.to_string()),
                    task.status
                        .map_or_else(|| "-".to_string(), |v| v.to_string().to_uppercase()),
                ];
                if verbose {
                    // If verbose, add all task cols to display table
                    cols.push(task.start_time.unwrap_or("-".to_string()));
                    cols.push(task.end_time.unwrap_or("-".to_string()));
                    cols.push(task.repeat.unwrap_or("-".to_string()));
                    cols.push(task.notes.unwrap_or("-".to_string()));
                }
                cols
            })
            .collect::<Vec<Vec<String>>>(),
        config,
    );

    table
        .seperate_cols(seperate_cols)
        .seperate_rows(seperate_rows)
        .to_string()
}

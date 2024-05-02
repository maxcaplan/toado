use table::AsciiTable;

pub mod table;

/// Format a vector of tasks as a string to be displayed to the user
pub fn format_task_list(tasks: Vec<toado::Task>, verbose: bool) -> String {
    let table = AsciiTable::from(
        tasks
            .into_iter()
            .map(|task| {
                let mut cols = vec![
                    task.id.to_string(),
                    task.name,
                    task.priority.to_string(),
                    task.status.to_string().to_uppercase(),
                ];
                if verbose {
                    cols.push(task.start_time.unwrap_or("-".to_string()));
                    cols.push(task.end_time.unwrap_or("-".to_string()));
                    cols.push(task.repeat.unwrap_or("-".to_string()));
                }
                cols
            })
            .collect::<Vec<Vec<String>>>(),
    );

    table.to_string()
}

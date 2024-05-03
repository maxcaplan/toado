use table::AsciiTable;

pub mod table;

/// Format a vector of tasks as a string to be displayed to the user
pub fn format_task_list(tasks: Vec<toado::Task>, verbose: bool) -> String {
    let table = AsciiTable::from(
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
    );

    table.seperate_cols(true).seperate_rows(false).to_string()
}

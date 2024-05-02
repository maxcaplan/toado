use table::AsciiTable;

pub mod table;

/// Format a vector of tasks as a string to be displayed to the user
pub fn format_task_list(tasks: Vec<toado::Task>) -> String {
    let table = AsciiTable::from(
        tasks
            .into_iter()
            .map(|task| {
                vec![
                    task.id.to_string(),
                    task.name,
                    task.priority.to_string(),
                    task.status.to_string().to_uppercase(),
                    task.start_time.unwrap_or("-".to_string()),
                    task.end_time.unwrap_or("-".to_string()),
                    task.repeat.unwrap_or("-".to_string()),
                ]
            })
            .collect::<Vec<Vec<String>>>(),
    );

    table.to_string()
}

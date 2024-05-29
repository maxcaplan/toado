use crate::formatting::table::AsciiTable;

/// Format a single project as a string to be displayed to the user
// pub fn format_project(_project: toado::Project) -> String {
//     String::from("")
// }

/// Format a vector of projects as a string to be displayed to the user
pub fn format_project_list(
    projects: Vec<toado::Project>,
    seperate_cols: bool,
    seperate_rows: bool,
    verbose: bool,
) -> String {
    // Create table from project vector
    let table = AsciiTable::from(
        projects
            .into_iter()
            .map(|project| {
                // Map project to vector of strings
                let mut cols = vec![
                    project
                        .id
                        .map_or_else(|| "-".to_string(), |v| v.to_string()),
                    project.name.unwrap_or("-".to_string()),
                    project.start_time.unwrap_or("-".to_string()),
                    project.end_time.unwrap_or("-".to_string()),
                ];

                if verbose {
                    cols.push(project.notes.unwrap_or("-".to_string()))
                }

                cols
            })
            .collect::<Vec<Vec<String>>>(),
    );

    table
        .seperate_cols(seperate_cols)
        .seperate_rows(seperate_rows)
        .to_string()
}

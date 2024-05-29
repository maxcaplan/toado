use clap::Parser;
use std::{env, fs, path::PathBuf, process};

mod commands;
mod flags;
mod formatting;

/// "The ships hung in the sky in much the same way that bricks don't."
fn main() {
    // Get CLI arguments
    let args = flags::Cli::parse();

    // Run the application and capture result
    let run = || -> Result<(), toado::Error> {
        // Get application directory
        let database_path = match init_database_path(args.file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to initialize application directory: {e}");
                return Err(e);
            }
        };

        // Open application server
        let app = match toado::Server::open(database_path) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Failed to create application server: {e}");
                return Err(e);
            }
        };

        // Init application database
        if let Err(e) = app.init() {
            eprintln!("Failed to initialize application server: {e}");
            return Err(e);
        };

        // If search term or command provided, execute and exit application
        if args.search.is_some() || args.command.is_some() {
            let res = {
                if let Some(search) = args.search {
                    handle_search(
                        flags::SearchArgs {
                            term: search,
                            task: args.task,
                            project: args.project,
                            verbose: args.verbose,
                        },
                        app,
                    )
                } else if let Some(command) = args.command {
                    handle_command(command, app)
                } else {
                    Ok(None)
                }
            };

            match res {
                Ok(Some(message)) => println!("{message}"),
                Err(e) => {
                    eprintln!("Failed to execute command: {e}");
                    return Err(e);
                }
                _ => {}
            };

            return Ok(());
        }

        // TODO: If no command provided, run TUI
        println!("toado");
        Ok(())
    };

    // If running the application results in error, terminate process
    if let Err(e) = run() {
        // If there is a source of the error, print to stderr
        if let Some(e) = e.source() {
            eprintln!("Caused by: {e}")
        }

        process::exit(1)
    }
}

/// Gets the path to the application database. If none is provieded, uses the default database file
/// location while ensuring the path exists
///
/// # Errors
///
/// Will return an error if getting the home directory fails, or if creating the default database
/// file location fails
fn init_database_path(path_string: Option<String>) -> Result<PathBuf, toado::Error> {
    if let Some(path_string) = path_string {
        let path = PathBuf::from(path_string);
        Ok(path)
    } else {
        let home_dir = env::var("HOME")?;
        let mut path = PathBuf::from(format!("{home_dir}/.local/share/toado/"));

        // Ensure application directory exists
        fs::create_dir_all(path.clone())?;

        // Append database filename to end of path
        path.push("database");
        Ok(path)
    }
}

/// Handle application commands from the CLI
///
/// # Errors
///
/// Will return an error if the executed command fails
fn handle_command(
    command: flags::Commands,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let message = match command {
        flags::Commands::Search(args) => handle_search(args, app)?,
        flags::Commands::Add(args) => handle_add(args, app)?,
        flags::Commands::Delete(args) => handle_delete(args, app)?,
        flags::Commands::Update(args) => handle_update(args, app)?,
        flags::Commands::Ls(args) => handle_ls(args, app)?,
        flags::Commands::Check(args) => handle_check(args, app)?,
        flags::Commands::Assign(args) => handle_assign(args, app)?,
    };

    Ok(message)
}

/// Handle the search command
///
/// # Errors
///
/// Will return an error if the task or project search fails
fn handle_search(
    args: flags::SearchArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        commands::search_tasks(args, app)
    } else {
        Err(Into::into("search is not implemented for projects"))
    }
}

/// Handle the add command
///
/// # Errors
///
/// Will return an error if the task or poject creation fails
fn handle_add(args: flags::AddArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        let (task_id, task_name) = commands::create_task(args, app)?;
        Ok(Some(format!(
            "Created task '{task_name}' with id '{task_id}'"
        )))
    } else {
        let (project_id, project_name) = commands::create_project(args, app)?;
        Ok(Some(format!(
            "Created project '{project_name}' with id '{project_id}'"
        )))
    }
}

/// Handle the delete command
///
/// # Errors
///
/// Will return an error if task or project deletion fails
fn handle_delete(
    args: flags::DeleteArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        match commands::delete_task(args, app)? {
            Some(id) => Ok(Some(format!("Deleted task with id {id}"))),
            None => Ok(None),
        }
    } else {
        match commands::delete_project(args, app)? {
            Some(id) => Ok(Some(format!("Deleted project with id {id}"))),
            None => Ok(None),
        }
    }
}

/// Handle the update command
///
/// # Errors
///
/// Will return an error if task or project updating fails
fn handle_update(
    args: flags::UpdateArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    Ok(Some(format!(
        "{} row(s) updated",
        if args.task || !args.project {
            commands::update_task(args, app)?
        } else {
            commands::update_project(args, app)?
        }
    )))
}

/// Handle the list command
///
/// # Errors
///
/// Will return an error if the task or project selection fails
fn handle_ls(args: flags::ListArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        commands::list_tasks(args, app)
    } else {
        commands::list_projects(args, app)
    }
}

/// Handle the check command
///
/// # Errors
///
/// Will return an error if task checking fails
fn handle_check(
    args: flags::CheckArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let (task_name, task_status) = commands::check_task(args, app)?;
    Ok(Some(format!(
        "Set '{task_name}' to {}",
        task_status.to_string().to_uppercase()
    )))
}

/// Handle the assign command
///
/// # Errors
///
/// Will return an error if assigning command fails
fn handle_assign(
    args: flags::AssignArgs,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let (pairs, action) = if !args.unassign {
        // Assign task(s)
        (
            if !args.no_select {
                commands::assign_multiple_tasks(args, app)?
            } else {
                vec![commands::assign_task(args, app)?]
            },
            "assigned to",
        )
    } else {
        // Unassign task(s)
        (
            if !args.no_select {
                commands::unassign_multiple_tasks(args, app)?
            } else {
                vec![commands::unassign_task(args, app)?]
            },
            "unassigned from",
        )
    };

    let message = pairs
        .into_iter()
        .map(|(task_name, project_name)| format!("'{task_name}' {action} '{project_name}'"))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(Some(message))
}

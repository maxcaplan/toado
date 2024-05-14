use clap::Parser;
use std::{env, fs, process};

mod commands;
mod flags;
mod formatting;

/// "The ships hung in the sky in much the same way that bricks don't."
fn main() {
    let args = flags::Cli::parse();

    let run = || -> Result<(), toado::Error> {
        // Get application directory
        let app_dir = match init_directory() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to initialize application directory: {e}");
                return Err(e);
            }
        };

        // Open application server
        let app = match toado::Server::open(&format!("{app_dir}/database")) {
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

    if let Err(e) = run() {
        if let Some(e) = e.source() {
            eprintln!("Caused by: {e}")
        }
        process::exit(1)
    }
}

/// Creates the directory for application files if one does not exist
///
/// # Errors
///
/// Will return an error if the creation of the application directories fails
fn init_directory() -> Result<String, toado::Error> {
    // Get user home directory
    let home_dir = env::var("HOME")?;
    let app_dir = format!("{home_dir}/.local/share/toado");

    // Create application directory if it doesn't exist
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
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
        flags::Commands::Ls(args) => handle_ls(args, app)?,
        flags::Commands::Check(args) => handle_check(args, app)?,
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
/// Will return an error if the task or poject creation fails
fn handle_add(args: flags::AddArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        let (task_id, task_name) = commands::create_task(args, app)?;
        Ok(Some(format!("Created task {task_name} with id {task_id}")))
    } else {
        Err(Into::into("project adding not implemented"))
    }
}

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
        Err(Into::into("project deletion not implemented"))
    }
}

/// Handle the list command
///
/// # Errors
/// Will return an error if the task or project selection fails
fn handle_ls(args: flags::ListArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        Ok(commands::list_tasks(&args, app)?)
    } else {
        Err(Into::into("task listing not implemented"))
    }
}

/// Handle the check command
fn handle_check(
    _args: flags::CheckArgs,
    _app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    Ok(None)
}

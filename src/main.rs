use clap::Parser;
use formatting::format_task_list;
use std::{env, fs, process};

mod commands;
mod flags;
mod formatting;

/// "The ships hung in the sky in much the same way that bricks don't."
fn main() {
    let args = flags::Cli::parse();

    let run = || -> Result<(), toado::Error> {
        let app_dir = match init_directory() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to initialize application directory: {e}");
                return Err(e);
            }
        };

        let app = match toado::Server::open(&format!("{app_dir}/database")) {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Failed to create application server: {e}");
                return Err(e);
            }
        };

        if let Err(e) = app.init() {
            eprintln!("Failed to initialize application server: {e}");
            return Err(e);
        };

        // If command provided, execute and exit application
        if let Some(command) = args.command {
            let message = match handle_command(command, app) {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("Failed to execute command: {e}");
                    return Err(e);
                }
            };

            // If command returns message, print to stdout
            if let Some(message) = message {
                println!("{message}")
            }

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
/// Will return an error if the creation of the application directories fails
pub fn init_directory() -> Result<String, toado::Error> {
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
/// Will return an error if the executed command fails
fn handle_command(
    command: flags::Commands,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let message = match command {
        flags::Commands::Add(args) => handle_add(args, app)?,
        flags::Commands::Delete(_args) => return Err(Into::into("deletion is not implemented")),
        flags::Commands::Ls(args) => handle_ls(args, app)?,
    };

    Ok(message)
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

/// Handle the list command
///
/// # Errors
/// Will return an error if the task or project selection fails
fn handle_ls(args: flags::ListArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        let tasks = commands::list_tasks(&args, app)?;
        Ok(Some(format_task_list(tasks, args.verbose)))
    } else {
        Err(Into::into("task listing not implemented"))
    }
}

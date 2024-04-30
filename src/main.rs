use clap::Parser;
use std::{env, fs, process};

mod flags;

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
pub fn init_directory() -> Result<String, toado::Error> {
    // Get user home directory
    let home_dir = env::var("HOME")?;
    let app_dir = format!("{home_dir}/.local/share/toado");

    // Create application directory if it doesn't exist
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

/// Handle application commands from the CLI
fn handle_command(
    command: flags::Commands,
    app: toado::Server,
) -> Result<Option<String>, toado::Error> {
    let message = match command {
        flags::Commands::Add(args) => on_add(args, app)?,
        flags::Commands::Delete(args) => Some(format!("Delete task with name: {}", args.name)),
        flags::Commands::List(_args) => Some("List tasks".to_string()),
    };

    Ok(message)
}

fn on_add(args: flags::AddArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    if args.task || !args.project {
        app.add_task(toado::AddTaskArgs {
            name: args.name.unwrap_or("unset".to_string()),
            priority: 0,
            status: toado::ItemStatus::Incomplete,
            start_time: "2024-05-05".to_string(),
            end_time: None,
        })
        .map(|id| Some(format!("Created new task with id `{id}`")))
    } else {
        Err(Into::into("Project adding not implemented"))
    }
}

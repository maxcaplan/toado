use clap::Parser;
use std::{env, fs, process};

mod flags;

fn main() {
    let args = flags::Cli::parse();

    let app_dir = init_directory().unwrap_or_else(|e| {
        eprintln!("Failed to initialize application directory: {e}");
        process::exit(1)
    });

    let app = match toado::Server::open(&format!("{app_dir}/database")) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to create application server: {e}");
            process::exit(1)
        }
    };

    app.init().unwrap_or_else(|e| {
        eprintln!("Failed to initialize application server: {e}");
        process::exit(1)
    });

    // If command provided, execute and exit application
    if let Some(command) = args.command {
        if let Some(message) = handle_command(command, app).unwrap_or_else(|e| {
            eprintln!("Failed to execute application command: {e}");
            process::exit(1)
        }) {
            // If command returned a message, print to stdout
            println!("{message}")
        }

        return;
    }

    println!("toado");
}

/// Creates the directory for application files if one does not exist
fn init_directory() -> Result<String, toado::Error> {
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
    app.add_task(toado::AddTaskArgs {
        name: args.name.unwrap_or("unset".to_string()),
        priority: 0,
        status: toado::ItemStatus::Incomplete,
        start_time: "2024-05-05".to_string(),
        end_time: None,
    })
    .map(|id| Some(format!("Created new task with id `{id}`")))
}

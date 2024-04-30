use std::{env, fs, process};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the toado database file
    #[arg(short, long)]
    file: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item
    Add(AddArgs),
    /// Remove an item
    Delete(DeleteArgs),
    /// Display a list of items
    List(ListArgs),
}

#[derive(Args)]
struct AddArgs {
    /// Name of item to add
    name: Option<String>,
    /// Add new task (default behaviour)
    #[arg(short, long)]
    task: bool,
    /// Add new project
    #[arg(short, long)]
    project: bool,
}

#[derive(Args)]
struct DeleteArgs {
    /// Name of item to delete
    name: String,
    /// Delete task (default behaviour)
    #[arg(short, long)]
    task: bool,
    /// Delete project
    #[arg(short, long)]
    project: bool,
}

#[derive(Args)]
struct ListArgs {
    /// List tasks (default behaviour)
    #[arg(short, long)]
    task: bool,
    /// List projects
    #[arg(short, long)]
    project: bool,
}

fn main() {
    let args = Cli::parse();

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
fn handle_command(command: Commands, app: toado::Server) -> Result<Option<String>, toado::Error> {
    let message = match command {
        Commands::Add(args) => on_add(args, app)?,
        Commands::Delete(args) => Some(format!("Delete task with name: {}", args.name)),
        Commands::List(_args) => Some("List tasks".to_string()),
    };

    Ok(message)
}

fn on_add(args: AddArgs, app: toado::Server) -> Result<Option<String>, toado::Error> {
    app.add_task(toado::AddTaskArgs {
        name: args.name.unwrap_or("unset".to_string()),
        priority: 0,
        status: toado::ItemStatus::Incomplete,
        start_time: "2024-05-05".to_string(),
        end_time: None,
    })
    .map(|id| Some(format!("Created new task with id `{id}`")))
}

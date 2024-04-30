use std::{env, fs, process};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the todo database file
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
    let app_dir = init_directory().unwrap_or_else(|e| {
        eprintln!("Failed to initialize application directory: {e}");
        process::exit(1)
    });

    println!("{app_dir}")
}

/// Creates the directory for application files if one does not exist
fn init_directory() -> Result<String, toado::Error> {
    // Get user home directory
    let home_dir = env::var("HOME")?;
    let app_dir = format!("{home_dir}/.local/share/todo_rs");

    // Create application directory if it doesn't exist
    fs::create_dir_all(&app_dir)?;
    Ok(app_dir)
}

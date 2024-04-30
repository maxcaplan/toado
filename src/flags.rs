use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the toado database file
    #[arg(short, long)]
    pub file: Option<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new item
    Add(AddArgs),
    /// Remove an item
    Delete(DeleteArgs),
    /// Display a list of items
    List(ListArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// Name of item to add
    pub name: Option<String>,
    /// Add new task (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Add new project
    #[arg(short, long)]
    pub project: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Name of item to delete
    pub name: String,
    /// Delete task (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Delete project
    #[arg(short, long)]
    pub project: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// List tasks (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// List projects
    #[arg(short, long)]
    pub project: bool,
}

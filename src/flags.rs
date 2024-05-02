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
    Ls(ListArgs),
}

#[derive(Args)]
pub struct AddArgs {
    /// Add new task (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Add new project
    #[arg(short, long)]
    pub project: bool,
    /// Name of item
    pub name: Option<String>,
    /// Priority of item
    #[arg(short, long)]
    pub item_priority: Option<u64>,
    /// Start time of item
    #[arg(short, long)]
    pub start_time: Option<String>,
    /// End time of item
    #[arg(short, long)]
    pub end_time: Option<String>,
    /// Notes to add to item
    #[arg(short, long)]
    pub notes: Option<String>,
    /// Repetition of item (Only applies to tasks)
    #[arg(short, long)]
    pub repeat: Option<String>,
    /// Skip optional fields
    #[arg(short, long)]
    pub optional: bool,
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
    /// Display all item information
    #[arg(short, long)]
    pub verbose: bool,
}

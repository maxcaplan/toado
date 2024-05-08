//! Toado cli flags
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Search term for item
    pub search: Option<String>,
    /// Execute search for tasks (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Execute search for projects
    #[arg(short, long)]
    pub project: bool,
    /// List all item information
    #[arg(short, long)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Path to database file
    #[arg(short, long)]
    pub file: Option<String>,
}

/// Application subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Search for items
    Search(SearchArgs),
    /// Add a new item
    Add(AddArgs),
    /// Remove an item
    Delete(DeleteArgs),
    /// Display a list of items
    Ls(ListArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    /// Search term for item
    pub term: String,
    /// Search for tasks (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Search for projects
    #[arg(short, long)]
    pub project: bool,
    /// List all item information
    #[arg(short, long)]
    pub verbose: bool,
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
    /// List item order
    pub order_by: Option<toado::OrderBy>,
    /// List tasks (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// List projects
    #[arg(short, long)]
    pub project: bool,
    /// List all item information
    #[arg(short, long)]
    pub verbose: bool,
    /// List in ascending order
    #[arg(short, long)]
    pub asc: bool,
    /// List in descending order
    #[arg(short, long)]
    pub desc: bool,
    /// Limit the number of items listed
    #[arg(short, long)]
    pub limit: Option<usize>,
    /// Offset start of list
    #[arg(short, long)]
    pub offset: Option<usize>,
    /// List all items
    #[arg(short, long)]
    pub full: bool,
}

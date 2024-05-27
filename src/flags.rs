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
    /// Update an item
    Update(UpdateArgs),
    /// Display a list of items
    Ls(ListArgs),
    /// Complete a task
    Check(CheckArgs),
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
    /// Repetition of item (tasks only)
    #[arg(short, long)]
    pub repeat: Option<String>,
    /// Skip optional fields
    #[arg(short, long)]
    pub optional: bool,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Search term for item to delete
    pub term: Option<String>,
    /// Delete task (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Delete project
    #[arg(short, long)]
    pub project: bool,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Search term for item to update
    pub term: Option<String>,
    /// Update task (default behaviour)
    #[arg(short, long)]
    pub task: bool,
    /// Update project
    #[arg(short, long)]
    pub project: bool,
    /// Update Name of item
    #[arg(short, long)]
    pub name: Option<String>,
    /// Update Priority of item
    #[arg(short, long, value_name = "PRIORITY")]
    pub item_priority: Option<u64>,
    /// Update Start time of item
    #[arg(short, long, value_name = "TIME|NULL")]
    pub start_time: Option<NullableString>,
    /// Update End time of item
    #[arg(short, long, value_name = "TIME|NULL")]
    pub end_time: Option<NullableString>,
    /// Update item notes
    #[arg(long, value_name = "NOTES|NULL")]
    pub notes: Option<NullableString>,
    /// Update Repetition of item (tasks only)
    #[arg(short, long, value_name = "REPEAT|NULL")]
    pub repeat: Option<NullableString>,
}

impl UpdateArgs {
    /// Returns true if any update value arguments are set for task values
    pub fn has_task_update_values(&self) -> bool {
        self.name.is_some()
            || self.item_priority.is_some()
            || self.start_time.is_some()
            || self.end_time.is_some()
            || self.notes.is_some()
            || self.repeat.is_some()
    }

    /// Returns true if any update value arguments are set for project values
    #[allow(dead_code)]
    pub fn has_project_update_values(&self) -> bool {
        self.name.is_some()
            || self.item_priority.is_some()
            || self.start_time.is_some()
            || self.end_time.is_some()
            || self.notes.is_some()
    }
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

#[derive(Args)]
pub struct CheckArgs {
    /// Search term for item to check
    pub term: Option<String>,
    /// Mark task as incomplete
    #[arg(short, long)]
    pub incomplete: bool,
}

/// CLI argument for a string value or Null
pub enum NullableString {
    Some(String),
    Null,
}

impl Clone for NullableString {
    fn clone(&self) -> Self {
        match self {
            Self::Some(value) => Self::Some(value.to_string()),
            Self::Null => Self::Null,
        }
    }
}

impl std::str::FromStr for NullableString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        if s == "null" {
            Ok(Self::Null)
        } else {
            Ok(Self::Some(s.to_string()))
        }
    }
}

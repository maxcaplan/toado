use queries::{
    AddProjectQuery, DeleteProjectQuery, DeleteTaskQuery, SelectProjectsQuery, UpdateProjectQuery,
};
pub use queries::{
    OrderBy, OrderDir, QueryCols, QueryConditions, RowLimit, SelectTasksQuery, UpdateAction,
    UpdateTaskCols, UpdateTaskQuery,
};
use std::{error, fmt, path::Path, usize};

use crate::queries::AddTaskQuery;

pub mod queries;

/// Toado application server
pub struct Server {
    /// SQLite database connection
    connection: rusqlite::Connection,
}

pub type Error = Box<dyn error::Error>;

impl Server {
    /// Opens a new toado app server with an sqlite database file.
    /// If the sqlite file does not exist, one is created at the path.
    ///
    /// # Errors
    ///
    /// Will return an error if the sqlite connection fails
    pub fn open<P>(file_path: P) -> Result<Server, Error>
    where
        P: AsRef<Path>,
    {
        let connection = rusqlite::Connection::open(file_path)?;

        Ok(Server { connection })
    }

    /// Initializes the application server by creating database tables
    ///
    /// # Errors
    ///
    /// Will return an error if the database initialization sql fails to execute
    pub fn init(&self) -> Result<(), Error> {
        self.connection.execute("PRAGMA foreign_keys = ON", ())?;

        self.connection.execute_batch(&format!(
            "BEGIN;
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS {}(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name TEXT NOT NULL,
                priority INTEGER NOT NULL,
                status INTEGER NOT NULL,
                start_time TEXT,
                end_time TEXT,
                repeat TEXT,
                notes TEXT
            );
            CREATE TABLE IF NOT EXISTS {}(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name TEXT NOT NULL,
                start_time TEXT,
                end_time TEXT,
                notes TEXT
            );
            CREATE TABLE IF NOT EXISTS {}(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            COMMIT;",
            Tables::Tasks,
            Tables::Projects,
            Tables::TaskAssignments
        ))?;

        Ok(())
    }

    /// Add a new task to the database. Returns id of added task
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn add_task(&self, args: AddTaskArgs) -> Result<i64, Error> {
        let query = AddTaskQuery::new(
            args.name,
            args.priority,
            args.start_time,
            args.end_time,
            args.repeat,
            args.notes,
        );

        self.connection.execute(&query.to_string(), ())?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Delete tasks from the database. Deletes all tasks matching query if is Some, if None deletes
    /// all tasks. Returns number of rows modified
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn delete_task(&self, condition: Option<String>) -> Result<u64, Error> {
        // Create delete query
        let query = DeleteTaskQuery::new(condition);
        // Execute query
        self.connection.execute(&query.to_string(), ())?;
        // Return number of rows deleted
        Ok(self.connection.changes())
    }

    /// Update tasks from the database with optional query. Only rows matching query will be
    /// updated. If no query provided, all rows in table will be updated. Returns the number of
    /// rows modified by update
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn update_task(
        &self,
        condition: Option<String>,
        args: UpdateTaskArgs,
    ) -> Result<u64, Error> {
        self.connection.execute(
            &UpdateTaskQuery {
                condition,
                name: args.name,
                priority: args.priority,
                status: args.status,
                start_time: args.start_time,
                end_time: args.end_time,
                repeat: args.repeat,
                notes: args.notes,
            }
            .to_string(),
            (),
        )?;

        Ok(self.connection.changes())
    }

    /// Select all tasks
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn select_tasks(
        &self,
        cols: QueryCols,
        condition: Option<String>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
        limit: Option<RowLimit>,
        offset: Option<usize>,
    ) -> Result<Vec<Task>, Error> {
        // Create query
        let query = SelectTasksQuery::new(cols, condition, order_by, order_dir, limit, offset);
        // Prepare query as statment
        let mut statment = self.connection.prepare(&query.to_string())?;

        // Map results from statment to data type
        let rows = statment.query_map((), |row| {
            // Convert status from i64 if value returned from query
            let status = match row.get::<&str, i64>("status") {
                Ok(value) => Some(ItemStatus::from(value)),
                Err(_) => None,
            };
            Ok(Task {
                id: row.get("id").ok(),
                name: row.get("name").ok(),
                priority: row.get("priority").ok(),
                status,
                start_time: row.get("start_time").ok(),
                end_time: row.get("end_time").ok(),
                repeat: row.get("repeat").ok(),
                notes: row.get("notes").ok(),
                projects: None,
            })
        })?;

        // Remove all empty rows, collect as vector of data and return
        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<Task>>())
    }

    /// Adds a new project to the application database
    ///
    /// # Errors
    ///
    /// Will return an error if execution of the query fails
    pub fn add_project(&self, args: AddProjectArgs) -> Result<i64, Error> {
        // Create query
        let query = AddProjectQuery::new(args.name, args.start_time, args.end_time, args.notes);
        // Execute query
        self.connection.execute(&query.to_string(), ())?;
        // Return id of inserted row
        Ok(self.connection.last_insert_rowid())
    }

    /// Updates a project in the application database
    ///
    /// # Errors
    ///
    /// Will return an error if the execution of the query fails
    pub fn update_project(
        &self,
        condition: Option<String>,
        name: UpdateAction<String>,
        start_time: UpdateAction<String>,
        end_time: UpdateAction<String>,
        notes: UpdateAction<String>,
    ) -> Result<u64, Error> {
        // Create query
        let query = UpdateProjectQuery::new(condition, name, start_time, end_time, notes);
        // Execute query
        self.connection.execute(&query.to_string(), ())?;
        // Return number of updated rows
        Ok(self.connection.changes())
    }

    /// Deletes one or more projects from the application database. If condition is None, deletes
    /// all projects (scary)
    ///
    /// # Errors
    ///
    /// Will return an error if the sql statment fails to execute
    pub fn delete_project(&self, condition: Option<String>) -> Result<u64, Error> {
        // Create delete query
        let query = DeleteProjectQuery::new(condition);
        // Execure query
        self.connection.execute(&query.to_string(), ())?;
        // Return number of deleted rows
        Ok(self.connection.changes())
    }

    /// Selects projects from the application database
    ///
    /// # Errors
    ///
    /// Will return an error if the sql statment fails to execute
    pub fn select_project(
        &self,
        cols: QueryCols,
        condition: Option<String>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
        limit: Option<RowLimit>,
        offset: Option<usize>,
    ) -> Result<Vec<Project>, Error> {
        // Create query
        let query = SelectProjectsQuery::new(cols, condition, order_by, order_dir, limit, offset);
        // Prepare query as statment
        let mut statment = self.connection.prepare(&query.to_string())?;

        // Map results from statment to data type
        let rows = statment.query_map((), |row| {
            Ok(Project {
                id: row.get("id").ok(),
                name: row.get("name").ok(),
                start_time: row.get("start_time").ok(),
                end_time: row.get("end_time").ok(),
                notes: row.get("notes").ok(),
                tasks: None,
            })
        })?;

        // Remove all empty rows, collect as vector of data and return
        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<Project>>())
    }

    /// Returns the total number of rows in a given table.
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn get_table_row_count(&self, table: Tables) -> Result<usize, Error> {
        Ok(self
            .connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), (), |row| {
                row.get(0)
            })?)
    }
}

/// Toado database tables
pub enum Tables {
    /// "tasks"
    Tasks,
    /// "projects"
    Projects,
    /// "task_assignments"
    TaskAssignments,
}

impl fmt::Display for Tables {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tasks => "tasks",
                Self::Projects => "projects",
                Self::TaskAssignments => "task_assignments",
            }
        )
    }
}

/// Task row data
pub struct Task {
    pub id: Option<i64>,
    /// Name of the task
    pub name: Option<String>,
    /// Priority value for task, higher is more important
    pub priority: Option<u64>,
    /// Completion status of task
    pub status: Option<ItemStatus>,
    /// Start time of the task in ISO 8601 format
    pub start_time: Option<String>,
    /// End time of the task in ISO 8601 format
    pub end_time: Option<String>,
    /// Determins whether and how the task repeats
    pub repeat: Option<String>,
    /// Notes for the task
    pub notes: Option<String>,
    /// List of projects the task is associate with
    pub projects: Option<Vec<Project>>,
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Task {
            id: self.id,
            name: self.name.clone(),
            priority: self.priority,
            status: self.status,
            start_time: self.start_time.clone(),
            end_time: self.end_time.clone(),
            repeat: self.repeat.clone(),
            notes: self.notes.clone(),
            projects: self.projects.clone(),
        }
    }
}

/// Arguments for adding a task to the database
pub struct AddTaskArgs {
    pub name: String,
    pub priority: u64,
    pub status: ItemStatus,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub repeat: Option<String>,
    pub notes: Option<String>,
}

/// Arguments for updating a task in the database
pub struct UpdateTaskArgs {
    pub name: UpdateAction<String>,
    pub status: UpdateAction<ItemStatus>,
    pub priority: UpdateAction<u64>,
    pub start_time: UpdateAction<String>,
    pub end_time: UpdateAction<String>,
    pub repeat: UpdateAction<String>,
    pub notes: UpdateAction<String>,
}

impl UpdateTaskArgs {
    pub fn update_status(status: ItemStatus) -> Self {
        UpdateTaskArgs {
            name: UpdateAction::None,
            priority: UpdateAction::None,
            status: UpdateAction::Some(status),
            start_time: UpdateAction::None,
            end_time: UpdateAction::None,
            repeat: UpdateAction::None,
            notes: UpdateAction::None,
        }
    }
}

/// Project row data
pub struct Project {
    /// Id of project
    pub id: Option<i64>,
    /// Name of project
    pub name: Option<String>,
    /// Start time of the project in ISO 8601 format
    pub start_time: Option<String>,
    /// End time of the project in ISO 8601 format
    pub end_time: Option<String>,
    /// Notes for the project
    pub notes: Option<String>,
    /// Tasks assigned to the project
    pub tasks: Option<Vec<Task>>,
}

impl Clone for Project {
    fn clone(&self) -> Self {
        Project {
            id: self.id,
            name: self.name.clone(),
            start_time: self.start_time.clone(),
            end_time: self.end_time.clone(),
            notes: self.notes.clone(),
            tasks: self.tasks.clone(),
        }
    }
}

/// Arguments for adding project to database
pub struct AddProjectArgs {
    pub name: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub notes: Option<String>,
}

/// Status of an item (ie. task or project)
#[derive(Clone, Copy)]
pub enum ItemStatus {
    Incomplete,
    Complete,
    Archived,
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Incomplete => "incomplete",
                Self::Complete => "complete",
                Self::Archived => "archived",
            }
        )
    }
}

// Implements u32 conversion for ItemStatus
impl From<ItemStatus> for u32 {
    fn from(value: ItemStatus) -> Self {
        match value {
            ItemStatus::Incomplete => 0,
            ItemStatus::Complete => 1,
            ItemStatus::Archived => 2,
        }
    }
}

// Implements Item status conversion for i64
impl From<i64> for ItemStatus {
    fn from(value: i64) -> Self {
        match value {
            0 => ItemStatus::Incomplete,
            1 => ItemStatus::Complete,
            3 => ItemStatus::Archived,
            _ => ItemStatus::Archived,
        }
    }
}

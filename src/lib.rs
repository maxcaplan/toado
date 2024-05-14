pub use queries::{OrderBy, OrderDir, QueryCols, QueryConditions, RowLimit, SelectTasksQuery};
use std::{collections::HashMap, error, fmt, usize};

pub mod queries;

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
    pub fn open(file_path: &str) -> Result<Server, Error> {
        let connection = rusqlite::Connection::open(file_path).map_err(|e| e.to_string())?;

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
                start_time TEXT NOT NULL,
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
        let mut map: HashMap<&str, String> = HashMap::from([
            ("name", args.name),
            ("priority", args.priority.to_string()),
            ("status", u32::from(args.status).to_string()),
        ]);

        if let Some(start_time) = args.start_time {
            map.insert("start_time", start_time);
        }

        if let Some(end_time) = args.end_time {
            map.insert("end_time", end_time);
        }

        if let Some(repeat) = args.repeat {
            map.insert("repeat", repeat);
        }

        if let Some(notes) = args.notes {
            map.insert("repeate", notes);
        }

        let (cols, vals): (Vec<&str>, Vec<String>) = map
            .into_iter()
            .map(|(key, val)| (key, format!("'{}'", val.trim()))) // Surrond values with single quotes (ex: 'val')
            .unzip();

        let sql_string = format!(
            "INSERT INTO tasks({}) VALUES({})",
            cols.join(", "),
            vals.join(", ")
        );

        self.connection.execute(&sql_string, ())?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Delete tasks from the database. Deletes all tasks matching query is Some, if None deletes
    /// all tasks. Returns number of rows modified
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn delete_task(&self, query: Option<String>) -> Result<u64, Error> {
        let mut sql_string = String::from("DELETE FROM tasks");

        if let Some(query) = query {
            sql_string.push_str(&format!(" WHERE {query}"))
        }
        sql_string.push(';');

        self.connection.execute(&sql_string, ())?;
        Ok(self.connection.changes())
    }

    /// Update tasks from the database with optional query. Only rows matching query will be
    /// updated. If no query provided, all rows in table will be updated
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn update_task<U, T>(
        &self,
        _update: Vec<UpdateCol<U>>,
        _query: Option<T>,
        _limit: RowLimit,
    ) -> Result<u64, Error>
    where
        U: fmt::Display,
        T: fmt::Display,
    {
        Ok(0)
    }

    /// Select all tasks
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    pub fn select_tasks(
        &self,
        cols: QueryCols,
        condtion: Option<String>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
        limit: Option<RowLimit>,
        offset: Option<usize>,
    ) -> Result<Vec<Task>, Error> {
        // `String` generic specified as it can't be infered when `conditions` is `None` :/
        self.execute_select_tasks(SelectTasksQuery::new(
            cols, condtion, order_by, order_dir, limit, offset,
        ))
    }

    /// Executes a select query for tasks
    ///
    /// # Errors:
    ///
    /// Will return an error if execution of the sql statment fails
    fn execute_select_tasks(&self, query: SelectTasksQuery) -> Result<Vec<Task>, Error> {
        let mut statment = self.connection.prepare(&query.to_string())?;

        let rows = statment.query_map((), |row| {
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

        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<Task>>())
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

/// Update action for a database column
pub enum UpdateCol<'a, T>
where
    T: fmt::Display,
{
    /// Update a column with a value
    Some(&'a str, T),
    /// Set a column to null
    Null(&'a str),
}

impl<'a, T> fmt::Display for UpdateCol<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Some(col, val) => format!("{col} = {val}"),
                Self::Null(col) => format!("{col} = NULL"),
            }
        )
    }
}

// Task row data
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
    pub projects: Option<Vec<String>>,
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
    /// Name of the task
    pub name: String,
    /// Priority value for task, higher is more important
    pub priority: u64,
    /// Completion status of task
    pub status: ItemStatus,
    /// Start time of the task in ISO 8601 format
    pub start_time: Option<String>,
    /// End time of the task in ISO 8601 format
    pub end_time: Option<String>,
    /// Determins whether and how the task repeats
    pub repeat: Option<String>,
    /// Notes for the task
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

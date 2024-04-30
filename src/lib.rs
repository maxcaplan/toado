use std::{
    collections::HashMap,
    error,
    fmt::{self},
};

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

        self.connection.execute_batch(
            "BEGIN;
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS tasks(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name TEXT NOT NULL,
                priority INTEGER NOT NULL,
                status INTEGER NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT
            );
            CREATE TABLE IF NOT EXISTS projects(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT
            );
            CREATE TABLE IF NOT EXISTS task_assignments(
                id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                task_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            COMMIT;",
        )?;

        Ok(())
    }

    /// Add a new task to the database. Returns id of added task
    pub fn add_task(&self, args: AddTaskArgs) -> Result<i64, Error> {
        let mut map: HashMap<&str, String> = HashMap::from([
            ("name", args.name),
            ("priority", args.priority.to_string()),
            ("status", u32::from(args.status).to_string()),
            ("start_time", args.start_time),
        ]);

        if let Some(end_time) = args.end_time {
            map.insert("end_time", end_time);
        }

        let (cols, vals): (Vec<&str>, Vec<String>) = map
            .into_iter()
            .map(|(key, val)| (key, format!("'{val}'"))) // Surrond values with single quotes (ex: 'val')
            .unzip();

        let sql_string = format!(
            "INSERT INTO tasks({}) VALUES({})",
            cols.join(", "),
            vals.join(", ")
        );

        self.connection.execute(&sql_string, ())?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Delete tasks from the database by a query. Returns number of rows modified
    pub fn delete_task<T>(&self, query: QueryConditions<T>) -> Result<u64, Error>
    where
        T: fmt::Display,
    {
        let sql_string = format!("DELETE FROM tasks WHERE {};", String::from(query));
        self.connection.execute(&sql_string, ())?;

        Ok(self.connection.changes())
    }

    /// Select all tasks for a query
    pub fn select_tasks<T, E>(&self, query: QueryConditions<T>) -> Result<Vec<Task>, Error>
    where
        T: fmt::Display,
    {
        let sql_string = format!("SELECT * FROM tasks WHERE {};", String::from(query));
        let mut statment = self.connection.prepare(&sql_string)?;

        let rows = statment.query_map((), |row| {
            let status: i64 = row.get(3)?;
            Ok(Task {
                id: row.get(0)?,
                name: row.get(1)?,
                priority: row.get(2)?,
                status: ItemStatus::from(status),
                start_time: row.get(4)?,
                end_time: row.get(5).ok(),
                projects: None,
            })
        })?;

        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<Task>>())
    }
}

pub struct Task {
    pub id: i64,
    /// Name of the task
    pub name: String,
    /// Priority value for task, higher is more important
    pub priority: u64,
    /// Completion status of task
    pub status: ItemStatus,
    /// Start time of the task in ISO 8601 format
    pub start_time: String,
    /// End time of the task in ISO 8601 format
    pub end_time: Option<String>,
    /// List of projects the task is associate with
    pub projects: Option<Vec<String>>,
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
    pub start_time: String,
    /// End time of the task in ISO 8601 format
    pub end_time: Option<String>,
}

/// Status of an item (ie. task or project)
pub enum ItemStatus {
    Incomplete,
    Complete,
    Archived,
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

/// Database statment conditions
pub enum QueryConditions<'a, T>
where
    T: fmt::Display,
{
    Equal { col: &'a str, value: T },
    NotEqual { col: &'a str, value: T },
    GreaterThan { col: &'a str, value: T },
    LessThan { col: &'a str, value: T },
    GreaterThanOrEqual { col: &'a str, value: T },
    LessThanOrEqual { col: &'a str, value: T },
    Between { col: &'a str, values: (T, T) },
    Like { col: &'a str, value: T },
    In { col: &'a str, values: Vec<T> },
}

// Implements String conversion for QueryConditions
impl<'a, T> From<QueryConditions<'a, T>> for String
where
    T: fmt::Display,
{
    fn from(value: QueryConditions<T>) -> Self {
        match value {
            QueryConditions::Equal { col, value } => format!("{col} = {value}"),
            QueryConditions::NotEqual { col, value } => format!("{col} != {value}"),
            QueryConditions::GreaterThan { col, value } => format!("{col} > {value}"),
            QueryConditions::LessThan { col, value } => format!("{col} < {value}"),
            QueryConditions::GreaterThanOrEqual { col, value } => format!("{col} >= {value}"),
            QueryConditions::LessThanOrEqual { col, value } => format!("{col} <= {value}"),
            QueryConditions::Between { col, values } => {
                format!("{col} BETWEEN {} AND {}", values.0, values.1)
            }
            QueryConditions::Like { col, value } => format!("{col} LIKE {value}"),
            QueryConditions::In { col, values } => format!(
                "{col} IN ({})",
                values
                    .into_iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .join(", ") // Convert vector of values into string of format "a, b, c"
            ),
        }
    }
}

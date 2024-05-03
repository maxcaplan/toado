use std::{
    collections::HashMap,
    error,
    fmt::{self, Display},
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

    /// Delete tasks from the database by a query. Returns number of rows modified
    pub fn delete_task<T>(&self, query: QueryConditions<T>) -> Result<u64, Error>
    where
        T: fmt::Display,
    {
        let sql_string = format!("DELETE FROM tasks WHERE {};", query);
        self.connection.execute(&sql_string, ())?;

        Ok(self.connection.changes())
    }

    /// Select all tasks
    pub fn select_tasks(
        &self,
        cols: SelectCols,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
    ) -> Result<Vec<Task>, Error> {
        // `String` generic specified as it can't be infered when `conditions` is `None` :/
        self.execute_select_tasks::<String>(SelectTasksQuery::new(cols, None, order_by, order_dir))
    }

    /// Select all tasks for one or more given condition
    pub fn select_tasks_condition<T>(
        &self,
        cols: SelectCols,
        conditions: Vec<(QueryConditions<T>, Option<QueryOperators>)>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
    ) -> Result<Vec<Task>, Error>
    where
        T: fmt::Display,
    {
        self.execute_select_tasks(SelectTasksQuery::new(
            cols,
            Some(conditions),
            order_by,
            order_dir,
        ))
    }

    /// Executes a select query for tasks
    fn execute_select_tasks<T>(&self, query: SelectTasksQuery<T>) -> Result<Vec<Task>, Error>
    where
        T: fmt::Display,
    {
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
}

/// Task select query struct
struct SelectTasksQuery<'a, T: Display> {
    cols: SelectCols<'a>,
    conditions: Option<Vec<(QueryConditions<'a, T>, Option<QueryOperators>)>>,
    order_by: Option<OrderBy>,
    order_dir: Option<OrderDir>,
}

impl<'a, T: Display> SelectTasksQuery<'a, T> {
    fn new(
        cols: SelectCols<'a>,
        conditions: Option<Vec<(QueryConditions<'a, T>, Option<QueryOperators>)>>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
    ) -> SelectTasksQuery<'a, T> {
        SelectTasksQuery {
            cols,
            conditions,
            order_by,
            order_dir,
        }
    }
}

impl<'a, T: Display> fmt::Display for SelectTasksQuery<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut query_string = format!("SELECT {} FROM {}", self.cols, Tables::Tasks);

        if let Some(conditions) = &self.conditions {
            // If select condtions provided, add to query string
            query_string.push_str(
                &conditions
                    .iter()
                    .map(|(condition, operator)| {
                        // Map conditions to string representations
                        let mut condition_string = condition.to_string();
                        if let Some(operator) = operator {
                            // If an operator is supplied, append to condition string
                            condition_string.push_str(&format!(" {}", operator));
                        };

                        condition_string
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
            );
        }

        // Default order by priority
        let order_by = self.order_by.unwrap_or(OrderBy::Priority);

        query_string.push_str(&format!(
            " ORDER BY {} {}",
            order_by,
            match self.order_dir {
                // Set order direction if provided, else use defaults
                Some(dir) => dir,
                None => match order_by {
                    OrderBy::Priority => OrderDir::Desc,
                    _ => OrderDir::Asc,
                },
            }
        ));

        write!(f, "{query_string};")
    }
}

/// Toado database tables
enum Tables {
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

/// What columns to return from a select statment.
pub enum SelectCols<'a> {
    All,
    Some(Vec<&'a str>),
}

impl fmt::Display for SelectCols<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::All => "*".to_string(),
                Self::Some(cols) => cols.join(", "),
            }
        )
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
impl<'a, T> Display for QueryConditions<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
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
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<String>>()
                        .join(", ") // Convert vector of values into string of format "a, b, c"
                ),
            }
        )
    }
}

/// Database statment conditional boolean logical operators
pub enum QueryOperators {
    And,
    Or,
}

// Implements String conversion for QueryOperators
impl fmt::Display for QueryOperators {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                QueryOperators::And => "AND",
                QueryOperators::Or => "OR",
            }
        )
    }
}

/// Table column to order selection by
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OrderBy {
    Id,
    Name,
    Priority,
    StartDate,
    EndDate,
}

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Id => "id",
                Self::Name => "name",
                Self::Priority => "priority",
                Self::StartDate => "start_date",
                Self::EndDate => "end_date",
            }
        )
    }
}

/// Direction of selection order.
/// Asc: smallest value to largest
/// Desc: Largest value to smallest
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OrderDir {
    Asc,
    Desc,
}

impl fmt::Display for OrderDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Asc => "ASC",
                Self::Desc => "DESC",
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

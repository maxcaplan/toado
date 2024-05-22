//! Database query utilites

use crate::Tables;
use std::fmt;

/// Columns to use in query
pub enum QueryCols<'a> {
    /// All columns
    All,
    /// Subset of row columns by name
    Some(Vec<&'a str>),
}

// Implements String conversion for QueryCols
impl fmt::Display for QueryCols<'_> {
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

/// Update action for a database column
pub enum UpdateAction<T>
where
    T: fmt::Display,
{
    /// Update column with a value
    Some(T),
    /// Set column to null
    Null,
    /// Don't update column
    None,
}

impl<T> UpdateAction<T>
where
    T: fmt::Display,
{
    fn map_from<U, F>(from: &UpdateAction<T>, f: F) -> UpdateAction<U>
    where
        U: fmt::Display,
        F: FnOnce(&T) -> U,
    {
        match from {
            Self::Some(x) => UpdateAction::Some(f(x)),
            Self::None => UpdateAction::None,
            Self::Null => UpdateAction::Null,
        }
    }

    /// Returns true if the UpdateAction value None
    fn is_none(&self) -> bool {
        matches!(&self, Self::None)
    }
    /// Create the sql update statment string for a given column.
    /// Avoid using this when the UpdateAction value is None
    fn to_statment(&self, col: &str) -> String {
        match &self {
            Self::Some(value) => format!("{col} = {value}"),
            Self::Null => format!("{col} = NULL"),
            Self::None => "".to_string(),
        }
    }
}

impl<T> From<Option<T>> for UpdateAction<T>
where
    T: fmt::Display,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Some(value),
            None => Self::None,
        }
    }
}

/// Table column to order selection by
#[derive(Clone, Copy, clap::ValueEnum)]
pub enum OrderBy {
    Id,
    Name,
    Priority,
    // TODO: These options cause an sql error
    // StartDate,
    // EndDate,
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
                // Self::StartDate => "start_date",
                // Self::EndDate => "end_date",
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

/// Defines the total number of rows to limit a query to
pub enum RowLimit {
    /// A set number of rows
    Limit(usize),
    /// No limit of rows
    All,
}

/// Data struct for updating task columns
pub struct UpdateTaskCols {
    /// Name of the task
    pub name: UpdateAction<String>,
    /// Priority value for task, higher is more important
    pub priority: UpdateAction<u64>,
    /// Completion status of task
    pub status: UpdateAction<crate::ItemStatus>,
    /// Start time of the task in ISO 8601 format
    pub start_time: UpdateAction<String>,
    /// End time of the task in ISO 8601 format
    pub end_time: UpdateAction<String>,
    /// Determins whether and how the task repeats
    pub repeat: UpdateAction<String>,
    /// Notes for the task
    pub notes: UpdateAction<String>,
}

impl UpdateTaskCols {
    /// Create a new UpdateTaskCols
    pub fn new(
        name: UpdateAction<String>,
        priority: UpdateAction<u64>,
        status: UpdateAction<crate::ItemStatus>,
        start_time: UpdateAction<String>,
        end_time: UpdateAction<String>,
        repeat: UpdateAction<String>,
        notes: UpdateAction<String>,
    ) -> Self {
        Self {
            name,
            priority,
            status,
            start_time,
            end_time,
            repeat,
            notes,
        }
    }

    /// Create a new UpdateTaskCols for updating just the status column
    pub fn status(status: crate::ItemStatus) -> Self {
        Self {
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

impl fmt::Display for UpdateTaskCols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Conditionaly pushes an update action on to a vector formatted as string
        fn push_action<T>(
            mut actions: Vec<String>,
            action: &UpdateAction<T>,
            col: &str,
        ) -> Vec<String>
        where
            T: fmt::Display,
        {
            if !action.is_none() {
                actions.push(action.to_statment(col))
            }

            actions
        }

        let mut actions: Vec<String> = Vec::new();

        actions = push_action(actions, &self.name, "name");
        actions = push_action(actions, &self.priority, "priority");
        actions = push_action(
            actions,
            &UpdateAction::map_from(&self.status, |val| u32::from(*val)), // Enum to int
            "status",
        );
        actions = push_action(actions, &self.start_time, "start_time");
        actions = push_action(actions, &self.end_time, "end_time");
        actions = push_action(actions, &self.repeat, "repeat");
        actions = push_action(actions, &self.notes, "notes");

        write!(f, "{}", actions.join(","))
    }
}

/// Functionality for adding sql query string parameters
trait Query {
    /// Takes an existing query string and appends condition, order, limit, and offset
    fn build_query_string(
        mut query_string: String,
        condition: &Option<String>,
        order_by: &Option<OrderBy>,
        order_dir: &Option<OrderDir>,
        limit: &Option<RowLimit>,
        offset: &Option<usize>,
    ) -> String {
        //
        // Query Conditions
        //
        if let Some(condition) = condition {
            // If select condtions provided, add to query string
            query_string.push_str(&format!(" WHERE {}", condition));
        }

        //
        // Query Order
        //

        // Default order by priority
        let order_by = order_by.unwrap_or(OrderBy::Priority);

        query_string.push_str(&format!(
            " ORDER BY {} {}",
            order_by,
            match order_dir {
                // Set order direction if provided, else use defaults
                Some(dir) => dir,
                None => match order_by {
                    OrderBy::Priority => &OrderDir::Desc,
                    _ => &OrderDir::Asc,
                },
            }
        ));

        //
        // Query Limit
        //
        match limit {
            Some(RowLimit::Limit(limit)) => query_string.push_str(&format!(" LIMIT {limit}")),
            Some(RowLimit::All) => {}
            None => query_string.push_str(" LIMIT 10"),
        }

        //
        // Query Offset
        //
        if limit.is_none()
            || limit
                .as_ref()
                .is_some_and(|limit| !matches!(limit, RowLimit::All))
        {
            if let Some(offset) = offset {
                query_string.push_str(&format!(" OFFSET {offset}"))
            }
        }

        query_string
    }
}

/// Task select query struct
pub struct SelectTasksQuery<'a> {
    cols: QueryCols<'a>,
    condition: Option<String>,
    order_by: Option<OrderBy>,
    order_dir: Option<OrderDir>,
    limit: Option<RowLimit>,
    offset: Option<usize>,
}

impl<'a> SelectTasksQuery<'a> {
    pub fn new(
        cols: QueryCols<'a>,
        condition: Option<String>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
        limit: Option<RowLimit>,
        offset: Option<usize>,
    ) -> Self {
        SelectTasksQuery {
            cols,
            condition,
            order_by,
            order_dir,
            limit,
            offset,
        }
    }
}

impl Query for SelectTasksQuery<'_> {}

impl fmt::Display for SelectTasksQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create basic select query string
        let query_string = Self::build_query_string(
            format!("SELECT {} FROM {}", self.cols, Tables::Tasks),
            &self.condition,
            &self.order_by,
            &self.order_dir,
            &self.limit,
            &self.offset,
        );

        write!(f, "{query_string};")
    }
}

/// Database query struct for task update queries
pub struct UpdateTaskQuery {
    update: UpdateTaskCols,
    condition: Option<String>,
}

impl UpdateTaskQuery {
    pub fn new(update: UpdateTaskCols, condition: Option<String>) -> Self {
        UpdateTaskQuery { update, condition }
    }
}

impl Query for UpdateTaskQuery {}

impl fmt::Display for UpdateTaskQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create basic update query string
        let mut query_string = format!("UPDATE {} SET {}", Tables::Tasks, self.update);

        // Append query conditions
        if let Some(condtition) = &self.condition {
            query_string.push_str(&format!(" WHERE {condtition}"));
        }

        // End query string
        query_string.push(';');

        write!(f, "{query_string};")
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
impl<'a, T> fmt::Display for QueryConditions<'a, T>
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

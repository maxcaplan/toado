//! Database query utilites

use std::fmt::{self};

pub use projects::*;
pub use tasks::*;

use crate::Tables;

mod projects;
mod tasks;

//
// Query Structs
//

/// Base database query trait
trait Query: fmt::Display {
    fn query_table(&self) -> crate::Tables;
}

/// Database addition query supertrait
trait AddQuery: Query + fmt::Display {
    /// Vector of key value pairs for query (ie. ("name", "lorem ipsum"))
    fn key_value_pairs(&self) -> KeyValuePairs;

    /// Returns keys and values as seperate list strings
    fn get_key_value_strings(&self) -> (String, String) {
        let (keys, values): (Vec<&str>, Vec<String>) = self.key_value_pairs().0.into_iter().unzip();
        let values: Vec<String> = values.into_iter().map(|v| quote_string(&v)).collect(); // Add quotes to
                                                                                          // values
        (keys.join(", "), values.join(", "))
    }

    /// Creates a query string from struct data
    fn build_query_string(&self) -> String {
        let (keys, values) = self.get_key_value_strings();
        format!(
            "INSERT INTO {}({keys}) VALUES({values});",
            self.query_table()
        )
    }
}

/// Database update query trait
trait UpdateQuery: Query + fmt::Display {
    type Action: fmt::Display;

    fn condition(&self) -> Option<&str>;
    fn update_cols(&self) -> UpdateCols<Self::Action>;

    fn build_query_string(&self) -> String {
        let mut query_string = format!("UPDATE {} SET {}", self.query_table(), self.update_cols());

        if let Some(condition) = self.condition() {
            query_string.push_str(&format!(" WHERE {condition};"));
        } else {
            query_string.push(';')
        }

        query_string
    }
}

/// Database delete query trait
trait DeleteQuery: Query + fmt::Display {
    /// Get the condition for selecting which row(s) to delete. If None, deletes all rows in table
    fn condition(&self) -> &Option<String>;

    /// Creates a query string from struct data
    fn build_query_string(&self) -> String {
        let mut query_string = format!("DELETE FROM {}", self.query_table());

        if let Some(condition) = self.condition() {
            query_string.push_str(&format!(" WHERE {condition};"))
        } else {
            query_string.push(';');
        }

        query_string
    }
}

/// Select query filters tuple type
type SelectFilters<'a> = (
    &'a Option<String>,   // Condition
    &'a Option<OrderBy>,  // Order by col
    &'a OrderBy,          // Default order by col
    &'a Option<OrderDir>, // Order direction
    &'a Option<RowLimit>, // Row limit
    &'a Option<usize>,    // Row offset
);

/// Database select query trait
trait SelectQuery<'a>: Query + fmt::Display {
    /// Get query filter values
    fn query_filters(&self) -> SelectFilters;

    fn select_cols(&self) -> &QueryCols<'a>;

    /// Appends selection filters to a query string
    fn append_filters(&self, mut query_string: String) -> String {
        let (condition, order_by, order_by_default, order_dir, limit, offset) =
            self.query_filters();

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
        let order_by = order_by.unwrap_or(*order_by_default);

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

        query_string.push(';');
        query_string
    }

    /// Creates a query string from struct data
    fn build_query_string(&self) -> String {
        let query_string = format!("SELECT {} FROM {}", self.select_cols(), self.query_table());
        self.append_filters(query_string)
    }
}

/// Database query for assigning a task to a project
pub struct AssignTaskQuery {
    task_id: i64,
    project_id: i64,
}

impl AssignTaskQuery {
    pub fn new(task_id: i64, project_id: i64) -> Self {
        Self {
            task_id,
            project_id,
        }
    }
}

impl Query for AssignTaskQuery {
    fn query_table(&self) -> crate::Tables {
        Tables::TaskAssignments
    }
}

impl AddQuery for AssignTaskQuery {
    fn key_value_pairs(&self) -> KeyValuePairs {
        KeyValuePairs(vec![
            ("task_id", self.task_id.to_string()),
            ("project_id", self.project_id.to_string()),
        ])
    }
}

impl fmt::Display for AssignTaskQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

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
#[derive(Clone, Copy)]
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
    /// Maps inner value T to U using mapping function F
    fn map<U, F>(self, f: F) -> UpdateAction<U>
    where
        U: fmt::Display,
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Some(value) => UpdateAction::Some(f(value)),
            Self::Null => UpdateAction::Null,
            Self::None => UpdateAction::None,
        }
    }

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
            Self::Some(value) => format!("{col} = '{value}'"),
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

impl From<String> for UpdateAction<String> {
    fn from(value: String) -> Self {
        if value.is_empty() {
            UpdateAction::Null
        } else {
            UpdateAction::Some(value)
        }
    }
}

/// Columns to update in an update query
struct UpdateCols<'a, T>(Vec<(&'a str, UpdateAction<T>)>)
where
    T: fmt::Display;

impl<T> fmt::Display for UpdateCols<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let actions: Vec<String> = self
            .0
            .iter()
            .filter(|col| !col.1.is_none())
            .map(|col| col.1.to_statment(col.0))
            .collect();

        write!(f, "{}", actions.join(", "))
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

pub struct KeyValuePairs<'a>(Vec<(&'a str, String)>);

impl<'a> KeyValuePairs<'a> {
    /// Push a key value pair to a vector of pairs if value is Some
    fn push_pairs_if_some(&mut self, key: &'a str, value: Option<String>) {
        if let Some(value) = value {
            self.0.push((key, value))
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

/// Surronds input str with single quote
fn quote_string(str: &str) -> String {
    format!("'{str}'")
}

//! Database query utilites

use std::fmt;

pub use projects::*;
pub use tasks::*;

mod projects;
mod tasks;

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

pub struct KeyValuePairs<'a>(Vec<(&'a str, String)>);

impl<'a> KeyValuePairs<'a> {
    /// Push a key value pair to a vector of pairs if value is Some
    fn push_pairs_if_some(&mut self, key: &'a str, value: Option<String>) {
        if let Some(value) = value {
            self.0.push((key, value))
        }
    }
}

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

/// Functionality for adding sql query string parameters
trait QueryFilters {
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

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

pub enum RowLimit {
    Limit(usize),
    All,
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

impl fmt::Display for SelectTasksQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Create basic select query string
        let mut query_string = format!("SELECT {} FROM {}", self.cols, Tables::Tasks);

        //
        // Query Conditions
        //
        if let Some(condition) = &self.condition {
            // If select condtions provided, add to query string
            query_string.push_str(&format!(" WHERE {}", condition));
        }

        //
        // Query Order
        //

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

        //
        // Query Limit
        //
        match self.limit {
            Some(RowLimit::Limit(limit)) => query_string.push_str(&format!(" LIMIT {limit}")),
            Some(RowLimit::All) => {}
            None => query_string.push_str(" LIMIT 10"),
        }

        //
        // Query Offset
        //
        if self.limit.is_none()
            || self
                .limit
                .as_ref()
                .is_some_and(|limit| !matches!(limit, RowLimit::All))
        {
            if let Some(offset) = self.offset {
                query_string.push_str(&format!(" OFFSET {offset}"))
            }
        }

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

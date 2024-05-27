use std::fmt::{self};

use crate::Tables;

use super::*;

//
// Add Query
//

/// Database query for adding a new project
pub struct AddProjectQuery {
    name: String,
    start_time: Option<String>,
    end_time: Option<String>,
    notes: Option<String>,
}

impl AddProjectQuery {
    pub fn new(
        name: String,
        start_time: Option<String>,
        end_time: Option<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            name,
            start_time,
            end_time,
            notes,
        }
    }
}

impl Query for AddProjectQuery {
    fn query_table(&self) -> crate::Tables {
        Tables::Projects
    }
}

impl fmt::Display for AddProjectQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

impl AddQuery for AddProjectQuery {
    fn key_value_pairs(&self) -> KeyValuePairs {
        // Create pairs vector with name key value pair
        let mut pairs = KeyValuePairs(vec![("name", self.name.clone())]);

        // Conditionally push optional values
        pairs.push_pairs_if_some("start_time", self.start_time.clone());
        pairs.push_pairs_if_some("end_time", self.end_time.clone());
        pairs.push_pairs_if_some("notes", self.notes.clone());

        pairs
    }
}

//
// Delete Query
//

pub struct DeleteProjectQuery {
    condition: Option<String>,
}

impl DeleteProjectQuery {
    pub fn new(condition: Option<String>) -> Self {
        Self { condition }
    }
}

impl Query for DeleteProjectQuery {
    fn query_table(&self) -> crate::Tables {
        Tables::Projects
    }
}

impl DeleteQuery for DeleteProjectQuery {
    fn condition(&self) -> &Option<String> {
        &self.condition
    }
}

impl fmt::Display for DeleteProjectQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

//
// Select Query
//

pub struct SelectProjectsQuery<'a> {
    cols: QueryCols<'a>,
    condition: Option<String>,
    order_by: Option<OrderBy>,
    order_dir: Option<OrderDir>,
    limit: Option<RowLimit>,
    offset: Option<usize>,
}

impl<'a> SelectProjectsQuery<'a> {
    pub fn new(
        cols: QueryCols<'a>,
        condition: Option<String>,
        order_by: Option<OrderBy>,
        order_dir: Option<OrderDir>,
        limit: Option<RowLimit>,
        offset: Option<usize>,
    ) -> Self {
        Self {
            cols,
            condition,
            order_by,
            order_dir,
            limit,
            offset,
        }
    }
}

impl Query for SelectProjectsQuery<'_> {
    fn query_table(&self) -> crate::Tables {
        crate::Tables::Projects
    }
}

impl<'a> SelectQuery<'a> for SelectProjectsQuery<'a> {
    fn query_filters(&self) -> SelectFilters {
        (
            &self.condition,
            &self.order_by,
            &OrderBy::Name,
            &self.order_dir,
            &self.limit,
            &self.offset,
        )
    }

    fn select_cols(&self) -> &QueryCols<'a> {
        &self.cols
    }
}

impl fmt::Display for SelectProjectsQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

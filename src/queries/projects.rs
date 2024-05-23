use std::fmt;

use super::{AddQuery, KeyValuePairs, Query};

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
        crate::Tables::Projects
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

use super::*;
use crate::Tables;
use std::fmt;

///
/// Add query
///

/// Database query for adding a new task
pub struct AddTaskQuery {
    name: String,
    priority: u64,
    start_time: Option<String>,
    end_time: Option<String>,
    repeat: Option<String>,
    notes: Option<String>,
}

impl AddTaskQuery {
    pub fn new(
        name: String,
        priority: u64,
        start_time: Option<String>,
        end_time: Option<String>,
        repeat: Option<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            name,
            priority,
            start_time,
            end_time,
            repeat,
            notes,
        }
    }
}

impl Query for AddTaskQuery {
    fn query_table(&self) -> crate::Tables {
        Tables::Tasks
    }
}

impl AddQuery for AddTaskQuery {
    fn key_value_pairs(&self) -> KeyValuePairs {
        let mut pairs = KeyValuePairs(vec![
            ("name", self.name.clone()),
            ("priority", self.priority.to_string()),
            ("status", crate::ItemStatus::Incomplete.to_string()),
        ]);

        pairs.push_pairs_if_some("start_time", self.start_time.clone());
        pairs.push_pairs_if_some("end_time", self.end_time.clone());
        pairs.push_pairs_if_some("repeat", self.repeat.clone());
        pairs.push_pairs_if_some("notes", self.notes.clone());

        pairs
    }
}

impl fmt::Display for AddTaskQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

///
/// Update Query
///

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

//
// Delete Query
//

pub struct DeleteTaskQuery {
    condition: Option<String>,
}

impl DeleteTaskQuery {
    pub fn new(condition: Option<String>) -> Self {
        Self { condition }
    }
}

impl Query for DeleteTaskQuery {
    fn query_table(&self) -> crate::Tables {
        Tables::Tasks
    }
}

impl DeleteQuery for DeleteTaskQuery {
    fn condition(&self) -> &Option<String> {
        &self.condition
    }
}

impl fmt::Display for DeleteTaskQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

//
// Select Query
//

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

impl Query for SelectTasksQuery<'_> {
    fn query_table(&self) -> crate::Tables {
        crate::Tables::Tasks
    }
}

impl<'a> SelectQuery<'a> for SelectTasksQuery<'a> {
    fn query_filters(&self) -> SelectFilters {
        (
            &self.condition,
            &self.order_by,
            &OrderBy::Priority,
            &self.order_dir,
            &self.limit,
            &self.offset,
        )
    }

    fn select_cols(&self) -> &QueryCols<'a> {
        &self.cols
    }
}

impl fmt::Display for SelectTasksQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.build_query_string())
    }
}

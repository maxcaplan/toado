use super::{OrderBy, OrderDir, Query, QueryCols, RowLimit, UpdateAction};
use crate::Tables;
use std::fmt;

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

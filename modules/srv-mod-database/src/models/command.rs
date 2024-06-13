use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::CUID2;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::schema::commands)]
pub struct Command {
    /// The unique identifier for the command (cuid2)
    pub id: String,
    /// Who ran the command
    pub ran_by: String,
    /// The command that was run
    pub command: String,
    /// The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
    /// was ran in the context of the global terminal emulator.
    pub session_id: String,
    /// The sequence_counter is a unique identifier for the command within the session_id.
    /// It is used to order the commands in the order they were ran.
    pub sequence_counter: i64,
    /// The command's output
    pub output: Option<String>,
    /// The command's exit code
    pub exit_code: Option<i32>,
    /// Soft delete for the `clear` command.
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Restore timestamp for the `history` command.
    pub restored_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The command's creation date
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The command's last update date
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::commands)]
pub struct CreateCommand {
    /// The unique identifier for the command (cuid2)
    pub id: String,
    /// Who ran the command
    pub ran_by: String,
    /// The command that was run
    pub command: String,
    /// The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
    /// was ran in the context of the global terminal emulator.
    pub session_id: String,
    /// The command's output
    pub output: Option<String>,
    /// The command's exit code
    pub exit_code: Option<i32>,
    /// Soft delete for the `clear` command.
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Restore timestamp for the `history` command.
    pub restored_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CreateCommand {
    pub fn new(ran_by: String, session_id: String) -> Self {
        Self {
            id: CUID2.create_id(),
            ran_by,
            command: "".to_string(),
            session_id,
            output: None,
            exit_code: None,
            deleted_at: None,
            restored_at: None,
        }
    }
}

/// A restore-able command represented with its full output
#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct FullHistoryRecord {
    sequence_counter: Option<i64>,
    command: String,
    output: Option<String>,
    exit_code: Option<i32>,
    ran_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// A compact command representation used to quickly and efficiently display a list of commands
#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct HistoryRecord {
    sequence_counter: Option<i64>,
    command: String,
    exit_code: Option<i32>,
    ran_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
}
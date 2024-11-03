use sea_orm::{prelude::DateTime, DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::TerminalHistory;

/// A restore-able command represented with its full output
#[derive(DerivePartialModel, FromQueryResult, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[sea_orm(entity = "TerminalHistory")]
pub struct FullHistoryRecord {
    /// The ordered sequence number of the command, this can be used to restore the commands in the
    /// same order
    sequence_counter: i64,
    /// The command that was ran
    command:          String,
    /// The full output of the command
    output:           Option<String>,
    /// The exit code of the command
    exit_code:        Option<i32>,
    /// The user who ran the command
    ran_by:           String,
    /// The time when the command was ran
    created_at:       DateTime,
}

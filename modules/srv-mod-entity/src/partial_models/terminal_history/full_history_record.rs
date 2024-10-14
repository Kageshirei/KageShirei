use sea_orm::{prelude::DateTime, DerivePartialModel, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::TerminalHistory;

/// A restore-able command represented with its full output
#[derive(DerivePartialModel, FromQueryResult, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[sea_orm(entity = "TerminalHistory")]
pub struct FullHistoryRecord {
    sequence_counter: i64,
    command:          String,
    output:           Option<String>,
    exit_code:        Option<i32>,
    ran_by:           String,
    created_at:       DateTime,
}

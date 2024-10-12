use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum AgentIntegrity {
    Untrusted = 0x00000000,
    Low = 0x00001000,
    Medium = 0x00002000,
    High = 0x00003000,
    System = 0x00004000,
    ProtectedProcess = 0x00005000,
}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i16", db_type = "SmallInteger")]
pub enum AgentIntegrity {
    Untrusted        = 0x00000000,
    Low              = 0x00001000,
    Medium           = 0x00002000,
    High             = 0x00003000,
    System           = 0x00004000,
    ProtectedProcess = 0x00005000,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "agent_field")]
pub enum AgentField {
    #[sea_orm(string_value = "agent")]
    Agent,
    #[sea_orm(string_value = "created_at")]
    CreatedAt,
    #[sea_orm(string_value = "cwd")]
    Cwd,
    #[sea_orm(string_value = "domain")]
    Domain,
    #[sea_orm(string_value = "hostname")]
    Hostname,
    #[sea_orm(string_value = "id")]
    Id,
    #[sea_orm(string_value = "integrity")]
    Integrity,
    #[sea_orm(string_value = "network_interfaces")]
    NetworkInterfaces,
    #[sea_orm(string_value = "operating_system")]
    OperatingSystem,
    #[sea_orm(string_value = "pid")]
    Pid,
    #[sea_orm(string_value = "ppid")]
    Ppid,
    #[sea_orm(string_value = "process_name")]
    ProcessName,
    #[sea_orm(string_value = "secret")]
    Secret,
    #[sea_orm(string_value = "server_secret")]
    ServerSecret,
    #[sea_orm(string_value = "signature")]
    Signature,
    #[sea_orm(string_value = "terminated_at")]
    TerminatedAt,
    #[sea_orm(string_value = "updated_at")]
    UpdatedAt,
    #[sea_orm(string_value = "username")]
    Username,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "command_status")]
pub enum CommandStatus {
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "running")]
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "filter_operation")]
pub enum FilterOperation {
    #[sea_orm(string_value = "contains")]
    Contains,
    #[sea_orm(string_value = "ends_with")]
    EndsWith,
    #[sea_orm(string_value = "equals")]
    Equals,
    #[sea_orm(string_value = "not_contains")]
    NotContains,
    #[sea_orm(string_value = "not_equals")]
    NotEquals,
    #[sea_orm(string_value = "starts_with")]
    StartsWith,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "log_level")]
pub enum LogLevel {
    #[sea_orm(string_value = "debug")]
    Debug,
    #[sea_orm(string_value = "error")]
    Error,
    #[sea_orm(string_value = "info")]
    Info,
    #[sea_orm(string_value = "trace")]
    Trace,
    #[sea_orm(string_value = "warning")]
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "logical_operator")]
pub enum LogicalOperator {
    #[sea_orm(string_value = "and")]
    And,
    #[sea_orm(string_value = "or")]
    Or,
}

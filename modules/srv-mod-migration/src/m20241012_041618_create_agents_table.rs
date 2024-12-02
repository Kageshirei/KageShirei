use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
};

use crate::sea_orm::{EnumIter, Iterable as _};

/// The integrity level of an agent
#[derive(DeriveIden)]
struct AgentIntegrity;

/// The possible integrity levels of an agent
#[derive(DeriveIden, EnumIter)]
enum AgentIntegrityVariants {
    /// The agent is untrusted
    Untrusted,
    /// The agent has low integrity
    Low,
    /// The agent has medium integrity
    Medium,
    /// The agent has high integrity
    High,
    /// The agent is protected by the system
    System,
    /// The agent is a protected process
    ProtectedProcess,
    /// The integrity provided is invalid
    #[expect(
        clippy::upper_case_acronyms,
        reason = "The integrity provided is invalid and must be easily recognizable in code"
    )]
    INVALID,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AgentIntegrity)
                    .values(AgentIntegrityVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Agent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Agent::Id).string_len(32).primary_key())
                    .col(string(Agent::OperativeSystem).not_null())
                    .col(string(Agent::Hostname).not_null())
                    .col(string_null(Agent::Domain))
                    .col(string(Agent::Username).not_null())
                    .col(json(Agent::NetworkInterfaces).default("[]"))
                    .col(big_integer(Agent::PID).not_null().default(0i64))
                    .col(big_integer(Agent::PPID).not_null().default(0i64))
                    .col(string(Agent::ProcessName).not_null())
                    .col(
                        enumeration(
                            Agent::Integrity,
                            Alias::new("agent_integrity"),
                            AgentIntegrityVariants::iter(),
                        )
                        .not_null(),
                    )
                    .col(string(Agent::CurrentWorkingDirectory).not_null())
                    .col(string(Agent::ServerSecret).not_null())
                    .col(string(Agent::Secret).not_null())
                    .col(string(Agent::Signature).not_null().unique_key())
                    .col(timestamp_null(Agent::TerminatedAt))
                    .col(timestamp(Agent::CreatedAt).not_null())
                    .col(timestamp(Agent::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Agent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden, EnumIter)]
pub enum Agent {
    /// The id of the agent
    Table,
    /// The id of the agent
    Id,
    /// The operating system of the agent
    #[sea_orm(iden = "operating_system")]
    OperativeSystem,
    /// The hostname of the agent
    Hostname,
    /// The domain of the agent
    Domain,
    /// The username of the agent
    Username,
    /// The network interfaces of the agent
    #[sea_orm(iden = "network_interfaces")]
    NetworkInterfaces,
    /// The process identifier of the agent
    PID,
    /// The parent process identifier of the agent
    PPID,
    /// The name of the process of the agent
    #[sea_orm(iden = "process_name")]
    ProcessName,
    /// The integrity level of the agent
    Integrity,
    /// The current working directory of the agent
    #[sea_orm(iden = "cwd")]
    CurrentWorkingDirectory,
    /// The server secret used to communicate with the agent
    #[sea_orm(iden = "server_secret")]
    ServerSecret,
    /// The secret associated with the agent
    Secret,
    /// The signature of the agent
    Signature,
    /// The time the agent was terminated
    #[sea_orm(iden = "terminated_at")]
    TerminatedAt,
    /// The time the agent was created
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    /// The time the agent was updated
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

pub type AgentFieldVariants = Agent;

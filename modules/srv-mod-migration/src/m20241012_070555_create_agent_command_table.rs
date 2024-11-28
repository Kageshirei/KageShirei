use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    extension::postgres::Type,
    m20241012_041618_create_agents_table::Agent,
    sea_orm::{EnumIter, Iterable as _},
};

/// The status of a command
#[derive(DeriveIden)]
struct CommandStatus;

/// The possible statuses of a command
#[derive(DeriveIden, EnumIter)]
enum CommandStatusVariants {
    /// The command is pending
    Pending,
    /// The command is running
    Running,
    /// The command is completed
    Completed,
    /// The command has failed
    Failed,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(CommandStatus)
                    .values(CommandStatusVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AgentCommand::Table)
                    .if_not_exists()
                    .col(string_len(AgentCommand::Id, 32).primary_key())
                    .col(string_len(AgentCommand::AgentId, 32))
                    .col(json(AgentCommand::Command))
                    .col(text_null(AgentCommand::Output))
                    .col(integer_null(AgentCommand::ExitCode))
                    .col(
                        enumeration(
                            AgentCommand::Status,
                            Alias::new("command_status"),
                            CommandStatusVariants::iter(),
                        )
                        .default("pending"),
                    )
                    .col(timestamp_null(AgentCommand::RetrievedAt))
                    .col(timestamp_null(AgentCommand::CompletedAt))
                    .col(timestamp_null(AgentCommand::FailedAt))
                    .col(timestamp(AgentCommand::CreatedAt))
                    .col(timestamp(AgentCommand::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_agent_command_agent_id")
                    .from(AgentCommand::Table, AgentCommand::AgentId)
                    .to(Agent::Table, Agent::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentCommand::Table).to_owned())
            .await
    }
}

/// The column definitions for the `agent_command` table + the table name
#[derive(DeriveIden)]
enum AgentCommand {
    /// The table name
    Table,
    /// The unique identifier of the command
    Id,
    /// The agent identifier
    AgentId,
    /// The command to execute
    Command,
    /// The output of the command
    Output,
    /// The exit code of the command
    ExitCode,
    /// The status of the command (running, completed, failed, pending, etc.)
    Status,
    /// The timestamp when the command was retrieved
    RetrievedAt,
    /// The timestamp when the command was completed
    CompletedAt,
    /// The timestamp when the command failed
    FailedAt,
    /// The timestamp when the command was created
    CreatedAt,
    /// The timestamp when the command was updated
    UpdatedAt,
}

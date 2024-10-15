use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    extension::postgres::Type,
    m20241012_041618_create_agents_table::Agent,
    sea_orm::{EnumIter, Iterable as _},
};

#[derive(DeriveIden)]
struct CommandStatus;

#[derive(DeriveIden, EnumIter)]
enum CommandStatusVariants {
    Pending,
    Running,
    Completed,
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

#[derive(DeriveIden)]
enum AgentCommand {
    Table,
    Id,
    AgentId,
    Command,
    Output,
    ExitCode,
    Status,
    RetrievedAt,
    CompletedAt,
    FailedAt,
    CreatedAt,
    UpdatedAt,
}

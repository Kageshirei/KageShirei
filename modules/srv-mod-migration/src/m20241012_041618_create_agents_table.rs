use crate::sea_orm::{ActiveEnum, DbBackend, EnumIter, Iterable, Schema};
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Agent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Agent::Id)
                            .string_len(32)
                            .primary_key()
                    )
                    .col(string(Agent::OperativeSystem).not_null())
                    .col(string(Agent::Hostname).not_null())
                    .col(string_null(Agent::Domain))
                    .col(string(Agent::Username).not_null())
                    .col(json(Agent::NetworkInterfaces).default("[]"))
                    .col(big_integer(Agent::PID).not_null().default(0i64))
                    .col(big_integer(Agent::PPID).not_null().default(0i64))
                    .col(string(Agent::ProcessName).not_null())
                    .col(
                        small_integer(Agent::Integrity).not_null()
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
    Table,
    Id,
    #[sea_orm(iden = "operating_system")]
    OperativeSystem,
    Hostname,
    Domain,
    Username,
    #[sea_orm(iden = "network_interfaces")]
    NetworkInterfaces,
    PID,
    PPID,
    #[sea_orm(iden = "process_name")]
    ProcessName,
    Integrity,
    #[sea_orm(iden = "cwd")]
    CurrentWorkingDirectory,
    #[sea_orm(iden = "server_secret")]
    ServerSecret,
    Secret,
    Signature,
    #[sea_orm(iden = "terminated_at")]
    TerminatedAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

pub type AgentFieldVariants = Agent;
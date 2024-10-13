use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    extension::postgres::Type,
    m20241012_035559_create_users_table::User,
    m20241012_041618_create_agents_table::AgentFieldVariants,
    sea_orm::{EnumIter, Iterable},
};

#[derive(DeriveIden)]
struct LogLevel;

#[derive(DeriveIden, EnumIter)]
enum LogLevelVariants {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(LogLevel)
                    .values(LogLevelVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Logs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Logs::Id).string_len(32).primary_key())
                    .col(
                        enumeration(
                            Logs::Level,
                            Alias::new("log_level"),
                            LogLevelVariants::iter(),
                        )
                            .not_null(),
                    )
                    .col(string(Logs::Title).not_null())
                    .col(string_null(Logs::Message))
                    .col(json_null(Logs::Extra))
                    .col(timestamp(Logs::CreatedAt).not_null())
                    .col(timestamp(Logs::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ReadLogs::Table)
                    .if_not_exists()
                    .col(string_len(ReadLogs::LogId, 32))
                    .col(string_len(ReadLogs::ReadBy, 32))
                    .col(timestamp(ReadLogs::ReadAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(ReadLogs::Table)
                    .name("pk_read_logs")
                    .col(ReadLogs::LogId)
                    .col(ReadLogs::ReadBy)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_read_logs_log_id")
                    .from(ReadLogs::Table, ReadLogs::LogId)
                    .to(Logs::Table, Logs::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_read_logs_read_by")
                    .from(ReadLogs::Table, ReadLogs::ReadBy)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReadLogs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Logs::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(LogLevel).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Logs {
    Table,
    Id,
    Level,
    Title,
    Message,
    Extra,
    #[sea_orm(ident = "created_at")]
    CreatedAt,
    #[sea_orm(ident = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ReadLogs {
    Table,
    #[sea_orm(ident = "log_id")]
    LogId,
    #[sea_orm(ident = "read_by")]
    ReadBy,
    #[sea_orm(ident = "read_at")]
    ReadAt,
}

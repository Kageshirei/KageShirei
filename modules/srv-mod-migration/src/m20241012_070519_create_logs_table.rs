use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    extension::postgres::Type,
    m20241012_035559_create_users_table::User,
    sea_orm::{EnumIter, Iterable as _},
};

/// The level of a log
#[derive(DeriveIden)]
struct LogLevel;

/// The possible levels of a log
#[derive(DeriveIden, EnumIter)]
enum LogLevelVariants {
    /// The log is an error
    Error,
    /// The log is a warning
    Warning,
    /// The log is an information
    Info,
    /// The log is a debug
    Debug,
    /// The log is a trace
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
                    .col(string_len(Logs::Id, 32).primary_key())
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
                    .col(string_len(ReadLogs::Id, 32).primary_key())
                    .col(string_len(ReadLogs::LogId, 32))
                    .col(string_len(ReadLogs::ReadBy, 32))
                    .col(timestamp(ReadLogs::ReadAt).not_null())
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

/// The columns for the `logs` table + the table name
#[derive(DeriveIden)]
pub enum Logs {
    /// The table name
    Table,
    /// The log id
    Id,
    /// The log level
    Level,
    /// The log title
    Title,
    /// The log message
    Message,
    /// Any log extra information as json
    Extra,
    /// The log creation timestamp
    #[sea_orm(ident = "created_at")]
    CreatedAt,
    /// The log update timestamp
    #[sea_orm(ident = "updated_at")]
    UpdatedAt,
}

/// The columns for the `read_logs` table + the table name
#[derive(DeriveIden)]
pub enum ReadLogs {
    /// The table name
    Table,
    /// The read log id
    Id,
    /// The log id
    #[sea_orm(ident = "log_id")]
    LogId,
    /// The user id that read the log
    #[sea_orm(ident = "read_by")]
    ReadBy,
    /// The timestamp when the log was read
    #[sea_orm(ident = "read_at")]
    ReadAt,
}

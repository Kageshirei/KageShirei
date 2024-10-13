use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AgentProfile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentProfile::Id)
                            .string_len(32)
                            .primary_key(),
                    )
                    .col(string(AgentProfile::Name).unique_key())
                    .col(timestamp_null(AgentProfile::KillDate))
                    .col(array_null(AgentProfile::WorkingHours, ColumnType::Time))
                    .col(
                        interval(AgentProfile::PollingInterval, None, None)
                            .not_null()
                            .default("1 minute"),
                    )
                    .col(
                        interval(AgentProfile::PollingJitter, None, None)
                            .not_null()
                            .default("10 seconds"),
                    )
                    .col(timestamp(AgentProfile::CreatedAt).not_null())
                    .col(timestamp(AgentProfile::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AgentProfile::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AgentProfile {
    Table,
    Id,
    Name,
    #[sea_orm(iden = "kill_date")]
    KillDate,
    #[sea_orm(iden = "working_hours")]
    WorkingHours,
    #[sea_orm(iden = "polling_interval")]
    PollingInterval,
    #[sea_orm(iden = "polling_jitter")]
    PollingJitter,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

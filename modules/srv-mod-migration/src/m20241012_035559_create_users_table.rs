use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    Password,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).string_len(32).primary_key())
                    .col(string(User::Username).not_null().unique_key())
                    .col(string(User::Password).not_null())
                    .col(timestamp(User::CreatedAt).not_null())
                    .col(timestamp(User::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

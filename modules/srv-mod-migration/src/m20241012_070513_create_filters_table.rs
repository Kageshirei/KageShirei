use crate::extension::postgres::Type;
use crate::m20241012_041618_create_agents_table::AgentFieldVariants;
use crate::m20241012_070459_create_agent_profiles_table::AgentProfile;
use crate::sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
struct AgentField;

#[derive(DeriveIden)]
struct FilterOperation;

#[derive(DeriveIden)]
struct LogicalOperator;

#[derive(DeriveIden, EnumIter)]
enum FilterOperationVariants {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
}

#[derive(DeriveIden, EnumIter)]
enum LogicalOperatorVariants {
    And,
    Or,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AgentField)
                    .values(AgentFieldVariants::iter())
                    .to_owned()
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(FilterOperation)
                    .values(FilterOperationVariants::iter())
                    .to_owned()
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(LogicalOperator)
                    .values(LogicalOperatorVariants::iter())
                    .to_owned()
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Filter::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Filter::Id)
                            .string_len(32)
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(Filter::AgentProfileId)
                            .string_len(32)
                            .not_null()
                    )
                    .col(enumeration(Filter::AgentField, Alias::new("agent_field"), AgentFieldVariants::iter()))
                    .col(enumeration(Filter::FilterOp, Alias::new("filter_operation"), FilterOperationVariants::iter()))
                    .col(json(Filter::Value).not_null())
                    .col(integer(Filter::Sequence).not_null().default(0i64))
                    .col(enumeration_null(Filter::NextHopRelation, Alias::new("logical_operator"), LogicalOperatorVariants::iter()))
                    .col(boolean(Filter::GroupingStart).not_null().default(false))
                    .col(boolean(Filter::GroupingEnd).not_null().default(false))
                    .col(timestamp(Filter::CreatedAt).not_null())
                    .col(timestamp(Filter::UpdatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_agent_profile_id")
                    .from(Filter::Table, Filter::AgentProfileId)
                    .to(AgentProfile::Table, AgentProfile::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Filter::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AgentField).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(FilterOperation).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(LogicalOperator).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Filter {
    Table,
    Id,
    #[sea_orm(iden = "agent_profile_id")]
    AgentProfileId,
    #[sea_orm(iden = "agent_field")]
    AgentField,
    #[sea_orm(iden = "filter_op")]
    FilterOp,
    Value,
    Sequence,
    #[sea_orm(iden = "next_hop_relation")]
    NextHopRelation,
    #[sea_orm(iden = "grouping_start")]
    GroupingStart,
    #[sea_orm(iden = "grouping_end")]
    GroupingEnd,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

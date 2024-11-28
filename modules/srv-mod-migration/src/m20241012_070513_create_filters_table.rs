use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    extension::postgres::Type,
    m20241012_041618_create_agents_table::AgentFieldVariants,
    m20241012_070459_create_agent_profiles_table::AgentProfile,
    sea_orm::{EnumIter, Iterable as _},
};

/// The possible fields of an agent
#[derive(DeriveIden)]
struct AgentField;

/// The possible filtering operations
#[derive(DeriveIden)]
struct FilterOperation;

/// The possible logical operator of filters
#[derive(DeriveIden)]
struct LogicalOperator;

/// The possible filtering operations
#[derive(DeriveIden, EnumIter)]
enum FilterOperationVariants {
    /// The filter operation equals
    Equals,
    /// The filter operation not equals
    NotEquals,
    /// The filter operation contains
    Contains,
    /// The filter operation not contains
    NotContains,
    /// The filter operation starts with
    StartsWith,
    /// The filter operation ends with
    EndsWith,
}

/// The possible logical operator of filters
#[derive(DeriveIden, EnumIter)]
enum LogicalOperatorVariants {
    /// The logical operator AND
    And,
    /// The logical operator OR
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
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(FilterOperation)
                    .values(FilterOperationVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(LogicalOperator)
                    .values(LogicalOperatorVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Filter::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Filter::Id).string_len(32).primary_key())
                    .col(
                        ColumnDef::new(Filter::AgentProfileId)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(enumeration(
                        Filter::AgentField,
                        Alias::new("agent_field"),
                        AgentFieldVariants::iter(),
                    ))
                    .col(enumeration(
                        Filter::FilterOp,
                        Alias::new("filter_operation"),
                        FilterOperationVariants::iter(),
                    ))
                    .col(json(Filter::Value).not_null())
                    .col(integer(Filter::Sequence).not_null().default(0i64))
                    .col(enumeration_null(
                        Filter::NextHopRelation,
                        Alias::new("logical_operator"),
                        LogicalOperatorVariants::iter(),
                    ))
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
                    .to_owned(),
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

/// The filters table columns + the table name
#[derive(DeriveIden)]
enum Filter {
    /// The table name
    Table,
    /// The primary key
    Id,
    /// The agent profile id
    #[sea_orm(iden = "agent_profile_id")]
    /// The agent field
    AgentProfileId,
    /// The agent field
    #[sea_orm(iden = "agent_field")]
    AgentField,
    /// The filter operation
    #[sea_orm(iden = "filter_op")]
    #[expect(
        clippy::enum_variant_names,
        reason = "The variant names are used as identifiers"
    )]
    FilterOp,
    /// The filter value
    Value,
    /// The filter sequence
    Sequence,
    /// The next hop relation (and, or)
    #[sea_orm(iden = "next_hop_relation")]
    NextHopRelation,
    /// Whether this filter is the start of a grouping (parenthesis open)
    #[sea_orm(iden = "grouping_start")]
    GroupingStart,
    /// Whether this filter is the end of a grouping (parenthesis close)
    #[sea_orm(iden = "grouping_end")]
    GroupingEnd,
    /// The created at timestamp
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    /// The updated at timestamp
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20241012_035559_create_users_table::User, m20241012_041618_create_agents_table::Agent};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TerminalHistory::Table)
                    .if_not_exists()
                    .col(string_len(TerminalHistory::Id, 32).primary_key())
                    .col(string_len(TerminalHistory::RanBy, 32))
                    .col(text(TerminalHistory::Command))
                    .col(string_null(TerminalHistory::SessionId))
                    .col(boolean(TerminalHistory::IsGlobal).default(true))
                    .col(text_null(TerminalHistory::Output))
                    .col(integer_null(TerminalHistory::ExitCode))
                    .col(big_integer(TerminalHistory::SequenceCounter).default(0))
                    .col(timestamp_null(TerminalHistory::DeletedAt))
                    .col(timestamp_null(TerminalHistory::RestoredAt))
                    .col(timestamp(TerminalHistory::CreatedAt))
                    .col(timestamp(TerminalHistory::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TerminalHistory::Table)
                    .name("u_ses_id_seq_counter")
                    .col(TerminalHistory::SessionId)
                    .col(TerminalHistory::SequenceCounter)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(TerminalHistory::Table)
                    .name("i_session_id")
                    .col(TerminalHistory::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_terminal_history_ran_by")
                    .from(TerminalHistory::Table, TerminalHistory::RanBy)
                    .to(User::Table, User::Id)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_terminal_history_session_id")
                    .from(TerminalHistory::Table, TerminalHistory::SessionId)
                    .to(Agent::Table, Agent::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        // Create a trigger to set the sequence_counter for each new row, based on the current max
        // sequence_counter for the session_id

        db.execute_unprepared(
            "
            create or replace function set_terminal_history_sequence_counter()
                returns trigger as
            $$
            declare
                current_max bigint;
            begin
                if new.session_id is null then
                    -- If session_id is NULL, calculate the max sequence_counter globally
                    select coalesce(max(sequence_counter), 0) + 1
                    into current_max
                    from terminal_history
                    where session_id is null;
                else
                    -- If session_id is not NULL, calculate the max sequence_counter for that session_id
                    select coalesce(max(sequence_counter), 0) + 1
                    into current_max
                    from terminal_history
                    where session_id = new.session_id;
                end if;

                -- Set the session_command_id for the new row
                new.sequence_counter = current_max;

                return new;
            end;
            $$ language plpgsql;
            ",
        )
        .await?;

        db.execute_unprepared(
            "
            create trigger before_insert_terminal_history
                before insert
                on terminal_history
                for each row
            execute function set_terminal_history_sequence_counter();
            ",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "
            drop trigger before_insert_terminal_history on terminal_history;
            ",
        )
        .await?;

        db.execute_unprepared(
            "
            drop function set_terminal_history_sequence_counter;
            ",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(TerminalHistory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TerminalHistory {
    Table,
    Id,
    #[sea_orm(iden = "ran_by")]
    RanBy,
    Command,
    #[sea_orm(iden = "session_id")]
    SessionId,
    #[sea_orm(iden = "is_global")]
    IsGlobal,
    Output,
    #[sea_orm(iden = "exit_code")]
    ExitCode,
    #[sea_orm(iden = "sequence_counter")]
    SequenceCounter,
    #[sea_orm(iden = "deleted_at")]
    DeletedAt,
    #[sea_orm(iden = "restored_at")]
    RestoredAt,
    #[sea_orm(iden = "created_at")]
    CreatedAt,
    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

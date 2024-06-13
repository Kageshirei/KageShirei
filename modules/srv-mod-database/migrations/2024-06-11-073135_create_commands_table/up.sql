-- Your SQL goes here
create table if not exists commands
(
    id               varchar(32) primary key,
    ran_by           varchar(32) not null references users (id),
    command          text        not null,
    -- The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
    -- was ran in the context of the global terminal emulator.
    session_id       varchar(32) not null,
    output           text,
    exit_code        integer,
    -- The sequence_counter is a unique identifier for the command within the session_id. It is used to order the commands
    -- in the order they were ran.
    sequence_counter bigint,
    -- Soft delete for the `clear` command.
    deleted_at       timestamptz,
    -- Restore timestamp for the `history` command.
    restored_at      timestamptz,
    created_at       timestamptz not null default current_timestamp,
    updated_at       timestamptz not null default current_timestamp,
    unique (session_id, sequence_counter)
);

create index if not exists idx_commands_session_id on commands (session_id);

create or replace function set_commands_sequence_counter()
    returns trigger as
$$
declare
    current_max bigint;
begin
    -- Get the current max session_command_id for this session_id
    select coalesce(max(sequence_counter), -1) + 1
    into current_max
    from commands
    where session_id = new.session_id;

    -- Set the session_command_id for the new row
    new.sequence_counter = current_max;

    return new;
end;
$$ language plpgsql;

create trigger before_insert_commands
    before insert
    on commands
    for each row
execute function set_commands_sequence_counter();
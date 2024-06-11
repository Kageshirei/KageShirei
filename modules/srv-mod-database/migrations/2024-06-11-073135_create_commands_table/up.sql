-- Your SQL goes here
create table if not exists commands (
    id          varchar(32) primary key,
    ran_by      varchar(32) not null references users (id),
    command     text        not null,
    -- The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
    -- was ran in the context of the global terminal emulator.
    session_id  varchar(32) not null,
    output      text,
    exit_code   integer,
    -- Soft delete for the `clear` command.
    deleted_at  timestamptz,
    -- Restore timestamp for the `history` command.
    restored_at timestamptz,
    created_at  timestamptz not null default current_timestamp,
    updated_at  timestamptz not null default current_timestamp
)
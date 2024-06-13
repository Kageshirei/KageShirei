-- Your SQL goes here
create table if not exists agents (
    id                varchar(32) primary key,
    operative_system  varchar(255) not null,
    hostname          varchar(255) not null,
    domain            varchar(255) not null,
    username          varchar(255) not null,
    ip                varchar(255) not null,
    -- the process id of the agent it is a u32 but in order to avoid two's
    -- complement errors (first bit is assigned to the sign of the number) we use bigint
    process_id        bigint       not null default 0,
    -- the parent process id of the agent it is a u32 but in order to avoid two's
    -- complement errors (first bit is assigned to the sign of the number) we use bigint
    parent_process_id bigint       not null default 0,
    process_name      varchar(255) not null,
    integrity_level int2        not null,
    cwd             text        not null,
    server_secret_key varchar(255) not null,
    secret_key        varchar(255) not null,
    signature         varchar(255) not null unique,
    created_at      timestamptz not null default current_timestamp,
    updated_at      timestamptz not null default current_timestamp
);

-- ref: https://www.postgresql.org/docs/current/indexes-types.html#INDEXES-TYPES-HASH
create index agents_signature_index on agents using hash (signature);
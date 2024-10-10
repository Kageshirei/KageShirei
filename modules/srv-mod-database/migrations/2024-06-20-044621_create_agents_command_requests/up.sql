-- Your SQL goes here
create type agent_command_status as enum (
    'pending',
    'running',
    'completed',
    'failed'
    );

create table if not exists agents_command_requests (
    id           varchar(32) primary key,
    agent_id     varchar(32)          not null references agents (id),
    command      jsonb                not null,
    output       text,
    status       agent_command_status not null default 'pending',
    retrieved_at timestamptz,
    completed_at timestamptz,
    failed_at    timestamptz,
    created_at   timestamptz          not null default current_timestamp,
    updated_at   timestamptz          not null default current_timestamp
);
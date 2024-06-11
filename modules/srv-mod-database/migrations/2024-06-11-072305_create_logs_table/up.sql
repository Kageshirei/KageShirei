-- Your SQL goes here
create type log_level as enum (
    'INFO',
    'WARN',
    'ERROR',
    'DEBUG',
    'TRACE'
    );

create table if not exists logs (
    id         varchar(32) primary key,
    level      log_level not null,
    message    text,
    title      text,
    extra      jsonb                default '{}'::jsonb,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
)
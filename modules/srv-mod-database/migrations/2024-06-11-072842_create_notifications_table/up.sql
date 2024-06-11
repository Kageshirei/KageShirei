-- Your SQL goes here
create table if not exists notifications (
    id         varchar(32) primary key,
    level      log_level not null,
    message    text      not null,
    title      text      not null,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
)
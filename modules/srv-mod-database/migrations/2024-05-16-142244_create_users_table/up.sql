create table if not exists users
(
    id         varchar(32) primary key,
    username   varchar(255) not null unique,
    password   varchar(255) not null,
    created_at timestamptz not null default current_timestamp,
    updated_at timestamptz not null default current_timestamp
);
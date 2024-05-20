create table if not exists users
(
    id         uuid                  default gen_random_uuid() primary key,
    username   varchar(255) not null unique,
    password   varchar(255) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
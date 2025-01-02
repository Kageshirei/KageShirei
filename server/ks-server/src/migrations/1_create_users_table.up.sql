create table if not exists users (
    id serial primary key,
    username text not null unique,
    password text not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now()
)
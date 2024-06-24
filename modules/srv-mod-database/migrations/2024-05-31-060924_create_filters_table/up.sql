-- Your SQL goes here
create type agent_fields as enum (
    'operative_system',
    'hostname',
    'domain',
    'username',
    'ip',
    'process_id',
    'parent_process_id',
    'process_name',
    'elevated',
    'server_secret_key',
    'secret_key',
    'signature'
    );

create type logical_operator as enum (
    'and',
    'or'
    );

create type filter_operator as enum (
    'equals',
    'not_equals',
    'contains',
    'not_contains',
    'starts_with',
    'ends_with'
    );

create table if not exists filters (
    id                varchar(32) primary key,
    agent_profile_id  varchar(32)     not null,
    agent_field       agent_fields    not null,
    filter_op         filter_operator not null,
    value             text            not null,
    sequence          int             not null,
    next_hop_relation logical_operator,
    grouping_start    boolean         not null default false,
    grouping_end      boolean         not null default false,
    created_at        timestamptz     not null default current_timestamp,
    updated_at        timestamptz     not null default current_timestamp,
    foreign key (agent_profile_id) references agent_profiles (id) on delete cascade
);
-- Your SQL goes here
create table if not exists agent_profiles (
    id               varchar(32) primary key,
    name             varchar(255) not null unique,
    -- unix timestamp of the date the agent must self-kill
    kill_date        bigint,
    -- array of unix timestamps of the hours the agent is allowed to work, in the format [start, end];
    -- if empty, the agent is allowed to work 24/7, if more than 2 elements, the agent is allowed to work in multiple
    -- time ranges in the same day always in pairs of [start, end], if the number of elements is odd, the last element
    -- is ignored
    working_hours    bigint[],
    polling_interval bigint,
    polling_jitter   bigint,
    created_at       timestamptz  not null default current_timestamp,
    updated_at       timestamptz  not null default current_timestamp
);
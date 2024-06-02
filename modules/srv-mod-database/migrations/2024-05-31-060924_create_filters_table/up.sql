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


----------------------------------------------------------------------------------------------------
---
--- The following commented sql statements shows an example of the insertion and retrieving logic
---
----------------------------------------------------------------------------------------------------
-- Insert a sample agent profile (if it does not already exist)
-- INSERT INTO agent_profiles (id, kill_date, created_at, updated_at, working_hours)
-- VALUES ('profile1', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ARRAY[0, 0]);
--
-- -- Insert a sample agent profile (if it does not already exist)
-- INSERT INTO agent_profiles (id, kill_date, created_at, updated_at, working_hours)
-- VALUES ('profile2', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ARRAY[0, 0]);
--
-- -- Insert a sample agent profile (if it does not already exist)
-- INSERT INTO agent_profiles (id, kill_date, created_at, updated_at, working_hours)
-- VALUES ('profile3', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ARRAY[0, 0]);
--
-- ---------------------------------------------------------------------------
-- -- Simple filter
-- INSERT INTO filters (id, agent_profile_id, agent_field, filter_op, value, sequence)
-- VALUES ('f1', 'profile1', 'ip', 'equals', '1.1.1.1', 1);
--
-- -- Complex filter -- (process_name starts_with "example" AND process_name ends_with ".exe") OR elevated equals true
-- -- Root expression: OR
-- INSERT INTO filters (id, agent_profile_id, logical_op, sequence)
-- VALUES ('f2', 'profile2', 'or', 1)
-- RETURNING id;
-- -- AND operator as a child of the OR operator
-- INSERT INTO filters (id, agent_profile_id, parent_id, logical_op, sequence)
-- VALUES ('f3', 'profile2', 'f2', 'and', 2)
-- RETURNING id;
-- -- A: process_name starts_with "example"
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f4', 'profile2', 'f3', 'process_name', 'starts_with', 'example', 3);
--
-- -- B: process_name ends_with ".exe"
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f5', 'profile2', 'f3', 'process_name', 'ends_with', '.exe', 4);
--
-- -- C: elevated equals true
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f6', 'profile2', 'f2', 'elevated', 'equals', 'true', 5);
--
-- -- Complex 1 level filter -- ip equals "1.1.1.1" AND elevated not_equals "true" OR process_name ends_with ".exe"
-- -- Step 2: Insert the root OR operator
-- INSERT INTO filters (id, agent_profile_id, logical_op, sequence)
-- VALUES ('f7', 'profile3', 'or', 1);
--
-- INSERT INTO filters (id, agent_profile_id, parent_id, logical_op, sequence)
-- VALUES ('f8', 'profile3', 'f7', 'and', 2);
--
-- -- Step 4: Insert the filters for ip and elevated as children of the AND operator
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f9', 'profile3', 'f8', 'ip', 'equals', '1.1.1.1', 3);
--
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f10', 'profile3', 'f8', 'elevated', 'not_equals', 'true', 4);
--
-- -- Step 5: Insert the filter for process_name as a child of the OR operator
-- INSERT INTO filters (id, agent_profile_id, parent_id, agent_field, filter_op, value, sequence)
-- VALUES ('f11', 'profile3', 'f7', 'process_name', 'ends_with', '.exe', 5);
--
-- --------------------------------------------------------
-- -- Query
--
-- WITH RECURSIVE filter_hierarchy AS (
--                                        -- Base case: Select the root filters with no parent
--                                        SELECT
--                                            id,
--                                            agent_profile_id,
--                                            parent_id,
--                                            agent_field,
--                                            filter_op,
--                                            value,
--                                            logical_op,
--                                            sequence,
--                                            1 AS level
--                                        FROM
--                                            filters
--                                        WHERE
--                                            agent_profile_id = 'profile3' AND parent_id IS NULL
--
--                                        UNION ALL
--
--                                        -- Recursive case: Select child filters and maintain their hierarchical level
--                                        SELECT
--                                            f.id,
--                                            f.agent_profile_id,
--                                            f.parent_id,
--                                            f.agent_field,
--                                            f.filter_op,
--                                            f.value,
--                                            f.logical_op,
--                                            f.sequence,
--                                            fh.level + 1 AS level
--                                        FROM
--                                            filters f
--                                                INNER JOIN
--                                            filter_hierarchy fh ON f.parent_id = fh.id
--                                    )
--
-- -- Select the filters in the correct order
-- SELECT
--     id,
--     agent_profile_id,
--     parent_id,
--     agent_field,
--     filter_op,
--     value,
--     logical_op,
--     sequence,
--     level
-- FROM
--     filter_hierarchy
-- ORDER BY
--     sequence,
--     level,
--     parent_id;

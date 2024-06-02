-- This file should undo anything in `up.sql`
drop table if exists filters;

drop type if exists agent_fields;

drop type if exists logical_operator;

drop type if exists filter_operator;
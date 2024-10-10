-- This file should undo anything in `up.sql`

-- Drop the trigger
drop trigger if exists before_insert_commands on commands;

-- Drop the trigger function
drop function if exists set_session_command_id();

drop table if exists commands;
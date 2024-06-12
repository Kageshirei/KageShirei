use std::fmt::Debug;

use anyhow::Result;
use serde::Serialize;

use srv_mod_database::Pool;

pub trait CommandHandler: Debug {
	/// Handle the command
	async fn handle_command(&self, session_id: &str, db_pool: Pool) -> Result<String>;
}

pub trait SerializableCommandHandler: CommandHandler + Serialize {}
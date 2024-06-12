use std::fmt::Debug;

use anyhow::Result;

use srv_mod_database::Pool;

pub trait CommandHandler: Debug {
	/// Handle the command
	fn handle_command(&self, session_id: &str, db_pool: Pool) -> impl std::future::Future<Output = Result<String>> + Send;
}


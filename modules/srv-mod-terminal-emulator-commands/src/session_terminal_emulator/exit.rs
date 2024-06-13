use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Handle the exit command
#[instrument]
pub async fn handle(_config: CommandHandlerArguments) -> anyhow::Result<String> {
	debug!("Terminal command received");

	// Signal the frontend terminal emulator to exit the terminal session
	Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__".to_string())
}
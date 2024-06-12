use tracing::{debug, instrument};

/// Handle the exit command
#[instrument]
pub async fn handle(_session_id: &str) -> anyhow::Result<String> {
	debug!("Terminal command received");

	// Signal the frontend terminal emulator to exit the terminal session
	Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__".to_string())
}
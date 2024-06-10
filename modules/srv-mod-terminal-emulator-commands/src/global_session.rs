use anyhow::Result;
use clap::{Parser, Subcommand};
use clap::builder::StyledStr;
use serde::Serialize;
use tracing::info;

use crate::command_handler::{CommandHandler, SerializableCommandHandler};
use crate::session_terminal_emulator::SessionTerminalEmulatorCommands;

mod session;

#[derive(Parser, Debug, PartialEq, Serialize)]
#[command(about, long_about = None, no_binary_name(true), bin_name = "")]
pub struct GlobalSessionTerminalEmulatorCommands {
	/// Turn debugging information on
	#[arg(
		short,
		long,
		action = clap::ArgAction::Count,
		global = true,
		long_help = r#"Turn debugging information on.

The more occurrences increase the verbosity level
- 0: No debugging information
- 1: Debugging information
- 2: Debug and trace information"#)]
	pub debug: u8,

	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Serialize)]
pub enum Commands {
	/// Clear the terminal screen
	#[serde(rename = "clear")]
	Clear,
	/// Exit the terminal session, closing the terminal emulator
	#[serde(rename = "exit")]
	Exit,
	Session,
}

impl CommandHandler for GlobalSessionTerminalEmulatorCommands {
	fn handle_command(&self) -> Result<String> {
		match &self.command {
			Commands::Clear => {
				info!("Terminal clear command received");

				// TODO: Implement the clear command hiding the output of previous commands (not dropping it by default)

				// Signal the frontend terminal emulator to clear the terminal screen
				Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__".to_string())
			}
			Commands::Exit => {
				info!("Terminal exit command received");

				// Signal the frontend terminal emulator to exit the terminal session
				Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__".to_string())
			}
			Commands::Session => {
				info!("Terminal session command received");

				// Signal the frontend terminal emulator to exit the terminal session
				Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_SESSION__".to_string())
			}
		}
	}
}

impl SerializableCommandHandler for GlobalSessionTerminalEmulatorCommands {}
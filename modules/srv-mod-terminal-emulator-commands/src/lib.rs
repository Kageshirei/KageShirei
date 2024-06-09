use anyhow::Result;
use clap::{Parser, Subcommand};
use clap::builder::StyledStr;
use serde::Serialize;
use tracing::info;

#[derive(Parser, Debug, PartialEq, Serialize)]
#[command(about, long_about = None, no_binary_name(true), bin_name = "")]
pub struct TerminalEmulatorCommands {
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
}

impl TerminalEmulatorCommands {
	pub fn from_raw(value: String) -> Result<TerminalEmulatorCommands, StyledStr> {
		let parsed_command = shellwords::split(value.as_str()).unwrap();
		Self::try_parse_from(parsed_command).map_err(|e| e.render())
	}

	pub fn handle_command(&self) -> Result<String> {
		match &self.command {
			Commands::Clear => {
				info!("Terminal clear command received");

				// TODO: Implement the clear command hiding the output of previous commands (not dropping it by default)

				// Signal the frontend terminal emulator to clear the terminal screen
				Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_CLEAR__".to_string())
			}
		}
	}
}
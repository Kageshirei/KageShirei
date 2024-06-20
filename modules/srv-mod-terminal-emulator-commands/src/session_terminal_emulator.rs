use clap::{Parser, Subcommand};
use serde::Serialize;

use crate::command_handler::{CommandHandler, CommandHandlerArguments};
use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;
use crate::session_terminal_emulator::history::TerminalSessionHistoryArguments;

pub(crate) mod clear;
pub(crate) mod exit;
pub(crate) mod history;
mod terminate;

#[derive(Parser, Debug, PartialEq, Serialize)]
#[command(about, long_about = None, no_binary_name(true), bin_name = "")]
pub struct SessionTerminalEmulatorCommands {
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
	Clear(TerminalSessionClearArguments),
	/// Exit the terminal session, closing the terminal emulator
	#[serde(rename = "exit")]
	Exit,
	/// Get the history of the terminal session and operate on it
	#[serde(rename = "history")]
	History(TerminalSessionHistoryArguments),
	/// Terminate the agent linked to the terminal session.
	///
	/// This action cannot be undone and will cause the immediate
	/// drop of any active connection
	#[serde(rename = "terminate")]
	Terminate,
}

impl CommandHandler for SessionTerminalEmulatorCommands {
	async fn handle_command(&self, config: CommandHandlerArguments) -> anyhow::Result<String> {
		match &self.command {
			Commands::Clear(args) => clear::handle(config, args).await,
			Commands::Exit => exit::handle(config).await,
			Commands::History(args) => history::handle(config, args).await,
			Commands::Terminate => terminate::handle(config).await,
		}
	}
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;

use crate::command_handler::{CommandHandler, CommandHandlerArguments};
use crate::global_session::make::TerminalSessionMakeArguments;
use crate::global_session::session::GlobalSessionTerminalSessionsArguments;
use crate::session_terminal_emulator::{clear, exit, history};
use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;
use crate::session_terminal_emulator::history::TerminalSessionHistoryArguments;

mod session;
mod make;

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
	Clear(TerminalSessionClearArguments),
	/// Exit the terminal session, closing the terminal emulator
	#[serde(rename = "exit")]
	Exit,
	/// Get the history of the terminal session and operate on it
	#[serde(rename = "history")]
	History(TerminalSessionHistoryArguments),
	/// List terminal sessions or open the terminal session for the provided hostnames
	#[serde(rename = "sessions")]
	Sessions(GlobalSessionTerminalSessionsArguments),
	/// Generate something
	#[serde(rename = "make")]
	Make(TerminalSessionMakeArguments),
}

impl CommandHandler for GlobalSessionTerminalEmulatorCommands {
	async fn handle_command(&self, config: CommandHandlerArguments) -> Result<String> {
		match &self.command {
			Commands::Clear(args) => clear::handle(config, args).await,
			Commands::Exit => exit::handle(config).await,
			Commands::History(args) => history::handle(config, args).await,
			Commands::Sessions(args) => session::handle(config, args).await,
			Commands::Make(args) => make::handle(config, args).await,
		}
	}
}

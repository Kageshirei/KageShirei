use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;

use srv_mod_database::Pool;

use crate::command_handler::CommandHandler;
use crate::global_session::session::GlobalSessionTerminalSessionsArguments;
use crate::session_terminal_emulator::{clear, exit, history};
use crate::session_terminal_emulator::clear::TerminalSessionClearArguments;
use crate::session_terminal_emulator::history::TerminalSessionHistoryArguments;

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
}

impl CommandHandler for GlobalSessionTerminalEmulatorCommands {
	async fn handle_command(&self, session_id: &str, db_pool: Pool) -> Result<String> {
		match &self.command {
			Commands::Clear(args) => clear::handle(session_id, db_pool, args).await,
			Commands::Exit => exit::handle(session_id).await,
			Commands::History(args) => history::handle(session_id, db_pool, args).await,
			Commands::Sessions(args) => session::handle(db_pool, args).await,
		}
	}
}

use clap::{Parser, Subcommand};
pub use clap::builder::StyledStr;
use serde::{Serialize, Serializer};

use srv_mod_database::Pool;

use crate::command_handler::{CommandHandler, SerializableCommandHandler};
use crate::global_session::GlobalSessionTerminalEmulatorCommands;
use crate::session_terminal_emulator::SessionTerminalEmulatorCommands;

pub mod global_session;
pub mod command_handler;
pub mod session_terminal_emulator;

#[derive(Debug, PartialEq)]
pub enum Command {
	SessionTerminalEmulatorCommands(SessionTerminalEmulatorCommands),
	GlobalSessionTerminalEmulatorCommands(GlobalSessionTerminalEmulatorCommands),
}

impl Serialize for Command {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer,
	{
		match self {
			Self::SessionTerminalEmulatorCommands(cmd) => cmd.serialize(serializer),
			Self::GlobalSessionTerminalEmulatorCommands(cmd) => cmd.serialize(serializer),
		}
	}
}

/// Evaluate the command and return the opaque result
macro_rules! make_result_from_cmd {
    ($cmd:expr, $variant:path) => {
        if let Ok(cmd) = $cmd {
            Ok(Box::new($variant(*cmd)))
        }
        else {
			Err($cmd.err().unwrap())
		}
    };
}


impl Command {
	/// Parse the command from the raw string to the specified type
	fn internal_parse<T>(value: &str) -> Result<Box<T>, StyledStr>
		where T: Parser {
		let parsed_command = shellwords::split(value).unwrap();
		T::try_parse_from(parsed_command)
			.map_err(|e| e.render())
			.map(|c| Box::new(c))
	}

	/// Parse the command from the raw string
	pub fn from_raw(session_id: &str, value: &str) -> Result<Box<Self>, StyledStr> {
		return match session_id {
			// if the session_id is "global", parse the command as a GlobalSessionTerminalEmulatorCommands
			"global" => {
				let cmd = Self::internal_parse::<GlobalSessionTerminalEmulatorCommands>(value);
				make_result_from_cmd!(cmd, Command::GlobalSessionTerminalEmulatorCommands)
			}
			// otherwise, parse the command as a SessionTerminalEmulatorCommands
			_ => {
				let cmd = Self::internal_parse::<SessionTerminalEmulatorCommands>(value);
				make_result_from_cmd!(cmd, Command::SessionTerminalEmulatorCommands)
			}
		};
	}
}

impl CommandHandler for Command {
	async fn handle_command(&self, session_id: &str, db_pool: Pool) -> anyhow::Result<String> {
		match self {
			Command::SessionTerminalEmulatorCommands(cmd) => cmd.handle_command(session_id, db_pool).await,
			Command::GlobalSessionTerminalEmulatorCommands(cmd) => cmd.handle_command(session_id, db_pool).await,
		}
	}
}

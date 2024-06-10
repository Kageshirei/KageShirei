use clap::{Parser, Subcommand};
pub use clap::builder::StyledStr;
use serde::{Serialize, Serializer};

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

impl Command {
	pub fn from_raw(session_id: Option<String>, value: String) -> Result<Box<Self>, StyledStr> {
		// if a session_id is provided, parse the command as a SessionTerminalEmulatorCommands
		return if session_id.is_some() {
			let parsed_command = shellwords::split(value.as_str()).unwrap();
			let cmd = SessionTerminalEmulatorCommands::try_parse_from(parsed_command)
				.map_err(|e| e.render())
				.map(|c| Box::new(c));

			// if the command is successfully parsed, return it
			if let Ok(cmd) = cmd {
				return Ok(Box::new(Command::SessionTerminalEmulatorCommands(*cmd)));
			}
			Err(cmd.err().unwrap())
		}
		// otherwise, parse the command as a GlobalSessionTerminalEmulatorCommands
		else {
			let parsed_command = shellwords::split(value.as_str()).unwrap();
			let cmd = GlobalSessionTerminalEmulatorCommands::try_parse_from(parsed_command)
				.map_err(|e| e.render())
				.map(|c| Box::new(c));

			// if the command is successfully parsed, return it
			if let Ok(cmd) = cmd {
				return Ok(Box::new(Command::GlobalSessionTerminalEmulatorCommands(*cmd)));
			}
			Err(cmd.err().unwrap())
		};
	}
}

impl CommandHandler for Command {
	fn handle_command(&self) -> anyhow::Result<String> {
		match self {
			Command::SessionTerminalEmulatorCommands(cmd) => cmd.handle_command(),
			Command::GlobalSessionTerminalEmulatorCommands(cmd) => cmd.handle_command(),
		}
	}
}

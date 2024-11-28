//! Terminal emulator commands handling logic for the terminal emulator on the client side

pub use clap::builder::StyledStr;
use clap::Parser;
use serde::{Serialize, Serializer};

use crate::{
    command_handler::{CommandHandler, CommandHandlerArguments},
    global_session::GlobalSessionTerminalEmulatorCommands,
    session_terminal_emulator::SessionTerminalEmulatorCommands,
};

pub mod command_handler;
pub mod global_session;
mod post_process_result;
pub mod session_terminal_emulator;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    SessionTerminalEmulatorCommands(SessionTerminalEmulatorCommands),
    GlobalSessionTerminalEmulatorCommands(GlobalSessionTerminalEmulatorCommands),
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[expect(clippy::pattern_type_mismatch, reason = "Cannot move out of self")]
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
    where
        T: Parser,
    {
        let parsed_command = shellwords::split(value).unwrap();
        T::try_parse_from(parsed_command)
            .map_err(|e| e.render())
            .map(|c| Box::new(c))
    }

    /// Parse the command from the raw string
    pub fn from_raw(session_id: &str, value: &str) -> Result<Box<Self>, StyledStr> {
        match session_id {
            // if the session_id is "global", parse the command as a GlobalSessionTerminalEmulatorCommands
            "global" => {
                let cmd = Self::internal_parse::<GlobalSessionTerminalEmulatorCommands>(value);
                make_result_from_cmd!(cmd, Command::GlobalSessionTerminalEmulatorCommands)
            },
            // otherwise, parse the command as a SessionTerminalEmulatorCommands
            _ => {
                let cmd = Self::internal_parse::<SessionTerminalEmulatorCommands>(value);
                make_result_from_cmd!(cmd, Command::SessionTerminalEmulatorCommands)
            },
        }
    }
}

impl CommandHandler for Command {
    async fn handle_command(&self, config: CommandHandlerArguments) -> Result<String, String> {
        #[expect(clippy::pattern_type_mismatch, reason = "Cannot move out of self")]
        match self {
            Self::SessionTerminalEmulatorCommands(cmd) => cmd.handle_command(config).await,
            Self::GlobalSessionTerminalEmulatorCommands(cmd) => cmd.handle_command(config).await,
        }
    }
}

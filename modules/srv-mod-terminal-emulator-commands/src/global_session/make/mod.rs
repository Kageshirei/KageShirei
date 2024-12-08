//! Global session terminal `make` commands

use clap::{Args, Subcommand};
use serde::Serialize;
use tracing::{debug, instrument};

use crate::{
    command_handler::CommandHandlerArguments,
    global_session::make::notification::TerminalSessionMakeNotificationArguments,
};

mod notification;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalSessionMakeArguments {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: MakeSubcommands,
}

/// Make something enumeration, a list of possible subcommands
#[derive(Subcommand, Debug, PartialEq, Eq, Serialize)]
#[allow(
    clippy::module_name_repetitions,
    reason = "Repetition in the name emphasizes that this enum represents distinct subcommands for the 'Make' \
              functionality."
)]
pub enum MakeSubcommands {
    /// Make a new notification and broadcast it to all connected clients
    #[serde(rename = "notification")]
    Notification(TerminalSessionMakeNotificationArguments),
}

/// Handle the history command
#[instrument]
pub async fn handle(config: CommandHandlerArguments, args: &TerminalSessionMakeArguments) -> Result<String, String> {
    debug!("Terminal command received");

    #[expect(clippy::pattern_type_mismatch, reason = "Cannot move out of self")]
    match &args.command {
        MakeSubcommands::Notification(args) => notification::handle(config.clone(), args).await,
    }
}

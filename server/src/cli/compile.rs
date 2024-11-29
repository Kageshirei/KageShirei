//! Compilation CLI arguments

use clap::{Args, Subcommand};

/// Compilation arguments
#[derive(Args, Debug, PartialEq, Eq)]
#[expect(clippy::module_name_repetitions, reason = "This is a speaking name")]
pub struct CompileArguments {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: CompileSubcommands,
}

/// Compilation subcommands
#[derive(Subcommand, Debug, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "This is a speaking name for the subcommands enum"
)]
pub enum CompileSubcommands {
    /// Compile the agent
    Agent,
    /// Compile the C2 GUI
    Gui,
}

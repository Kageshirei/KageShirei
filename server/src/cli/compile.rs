use clap::{Args, Subcommand};

/// Compilation arguments
#[derive(Args, Debug, PartialEq)]
pub struct CompileArguments {
	#[command(subcommand)]
	pub command: CompileSubcommands,
}

/// Compilation subcommands
#[derive(Subcommand, Debug, PartialEq)]
pub enum CompileSubcommands {
	/// Compile the agent
	Agent,
	/// Compile the C2 GUI
	Gui,
}

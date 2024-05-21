use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::cli::compile::CompileArguments;
use crate::cli::generate::GenerateArguments;
use crate::cli::run::RunArguments;

#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
	/// Turn debugging information on
	#[arg(short, long, action = clap::ArgAction::Count, global = true)]
	pub debug: u8,

	/// Path to the configuration file
	///
	/// Reads the configuration from the specified file, relative to the current working directory.
	#[arg(short, long, default_value = "config.json", global = true)]
	pub config: PathBuf,

	#[command(subcommand)]
	pub command: Commands,
}

/// First level server commands
#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
	/// Compile agent or C2 gui
	Compile(CompileArguments),
	/// Run the server
	Run(RunArguments),
	/// Generate strings or configuration files
	#[command(visible_alias = "make")]
	Generate(GenerateArguments),
}

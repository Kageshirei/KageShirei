use std::path::PathBuf;

use clap::Args;

/// Compilation arguments
#[derive(Args, Debug, PartialEq)]
pub struct RunArguments {
	/// Path to the configuration file
	///
	/// Reads the configuration from the specified file, relative to the current working directory.
	#[arg(short, long, default_value = "config.json")]
	pub config: PathBuf,
}
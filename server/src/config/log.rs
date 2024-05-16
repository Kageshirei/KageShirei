use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate, Clone)]
pub struct LogConfig {
	/// Configuration for the file logger
	pub file: FileLogConfig,
	/// Configuration for the console logger
	pub console: ConsoleLogConfig,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone)]
pub struct FileLogConfig {
	/// The path to the log file
	pub path: PathBuf,
	/// Whether to enable the logger
	pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone)]
pub struct ConsoleLogConfig {
	/// Whether to enable the logger
	pub enabled: bool,
}
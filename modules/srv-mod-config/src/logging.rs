use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct LogConfig {
	/// Configuration for the file logger
	pub file: FileLogConfig,
	/// Configuration for the console logger
	pub console: ConsoleLogConfig,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct FileLogConfig {
	/// The path to the log file
	pub path: PathBuf,
	/// Whether to enable the logger
	pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct ConsoleLogConfig {
	/// Whether to enable the logger
	pub enabled: bool,
	/// The format to use for the log output
	pub format: ConsoleLogFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ConsoleLogFormat {
	#[serde(rename = "pretty")]
	Pretty,
	#[serde(rename = "full")]
	#[default]
	Full,
	#[serde(rename = "compact")]
	Compact,
	#[serde(rename = "json")]
	Json,
}

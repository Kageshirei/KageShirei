use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct Config {
    /// Configuration for the file logger
    pub file:    FileConfig,
    /// Configuration for the console logger
    pub console: ConsoleConfig,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct FileConfig {
    /// The path to the log file
    pub path:    PathBuf,
    /// Whether to enable the logger
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct ConsoleConfig {
    /// Whether to enable the logger
    pub enabled: bool,
    /// The format to use for the log output
    pub format:  ConsoleFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum ConsoleFormat {
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

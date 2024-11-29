//! Base CLI arguments and subcommands

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::cli::{compile::CompileArguments, generate::GenerateArguments, run::RunArguments};

/// Base CLI arguments
#[derive(Parser, Debug, PartialEq, Eq)]
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

    /// The subcommand to run
    #[command(subcommand)]
    pub command: Commands,
}

/// First level server commands
#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Compile agent or C2 gui
    Compile(CompileArguments),
    /// Run the server
    Run(RunArguments),
    /// Generate strings or configuration files
    #[command(visible_alias = "make")]
    Generate(GenerateArguments),
}

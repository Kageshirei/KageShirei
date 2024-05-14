use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Commands,
}

/// First level server commands
#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Compile agent or C2 gui
    Compile(CompileArguments),
}

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
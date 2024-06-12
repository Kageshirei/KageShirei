use clap::{ArgAction, Args};
use serde::Serialize;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Serialize)]
pub struct GlobalSessionTerminalSessionArguments {
	/// List all available sessions
	#[arg(short, long)]
	pub list: bool,
	/// The list of session IDs to open terminal sessions for
	#[arg(action = ArgAction::Append)]
	pub ids: Vec<String>
}


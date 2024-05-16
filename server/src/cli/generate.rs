use clap::{Args, Subcommand};

/// Compilation arguments
#[derive(Args, Debug, PartialEq)]
pub struct GenerateArguments {
	#[command(subcommand)]
	pub command: GenerateSubcommands,
}

/// Compilation subcommands
#[derive(Subcommand, Debug, PartialEq)]
pub enum GenerateSubcommands {
	/// Generate a jwt key for the server to use for signing and verifying jwt tokens, the format is a 64 character string
	/// valid for HS512
	Jwt,
}
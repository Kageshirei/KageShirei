use clap::{Args, Subcommand};

use crate::cli::generate::certificate::GenerateCertificateArguments;
use crate::cli::generate::operator::GenerateOperatorArguments;

pub mod operator;
pub mod certificate;

/// Generate/make arguments
#[derive(Args, Debug, PartialEq)]
pub struct GenerateArguments {
	#[command(subcommand)]
	pub command: GenerateSubcommands,
}

/// Generate/make subcommands
#[derive(Subcommand, Debug, PartialEq)]
pub enum GenerateSubcommands {
	/// Generate a jwt key for the server to use for signing and verifying jwt tokens, the format is a 64 character string
	/// valid for HS512
	Jwt,
	/// Generate a new operator (aka user) for the server
	Operator(GenerateOperatorArguments),
	/// Generate a new self-signed tls certificate for the server
	Certificate(GenerateCertificateArguments),
}
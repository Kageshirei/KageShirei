//! Generate/make subcommands for the CLI

use clap::{Args, Subcommand};

use crate::cli::generate::{certificate::GenerateCertificateArguments, operator::GenerateOperatorArguments};

pub mod certificate;
pub mod operator;

/// Generate/make arguments
#[derive(Args, Debug, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "This is a speaking name for the subcommands enum"
)]
pub struct GenerateArguments {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: GenerateSubcommands,
}

/// Generate/make subcommands
#[derive(Subcommand, Debug, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "This is a speaking name for the subcommands enum"
)]
pub enum GenerateSubcommands {
    /// Generate a jwt key for the server to use for signing and verifying jwt tokens, the format is
    /// a 64 character string valid for HS512
    Jwt,
    /// Generate a new operator (aka user) for the server
    Operator(GenerateOperatorArguments),
    /// Generate a new self-signed tls certificate for the server
    Certificate(GenerateCertificateArguments),
    /// Generate dummy data for the server, this must be used only for testing purposes
    DummyData,
}

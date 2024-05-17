use clap::Args;

/// Generate/make operator arguments
#[derive(Args, Debug, PartialEq)]
pub struct GenerateOperatorArguments {
	/// The username for the operator
	pub username: String,
	/// The password for the operator
	pub password: String,
}
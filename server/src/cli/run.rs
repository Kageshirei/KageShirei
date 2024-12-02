//! Run command line arguments

use clap::Args;

/// Run arguments
#[derive(Args, Debug, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "The module is generally imported without full classification, the name avoids useless confusion"
)]
pub struct RunArguments;

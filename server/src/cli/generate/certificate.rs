use std::path::PathBuf;

use clap::{ArgAction::Append, Args};
use once_cell::sync::Lazy;
use srv_mod_database::diesel::internal::derives::multiconnection::chrono;

static NOT_BEFORE: Lazy<String> = Lazy::new(|| -> String {
    let duration = chrono::Utc::now().format("%Y-%m-%d").to_string();
    duration
});

static NOT_AFTER: Lazy<String> = Lazy::new(|| -> String {
    let duration = (chrono::Utc::now() + chrono::Duration::days(365))
        .format("%Y-%m-%d")
        .to_string();
    duration
});

/// Generate/make certificate arguments
#[derive(Args, Debug, PartialEq)]
pub struct GenerateCertificateArguments {
    /// Domain names for the certificate, can be specified multiple times
    #[arg(short = 'D', long, action = Append)]
    pub domains: Vec<String>,
    /// Validity period start date, in the format "YYYY-MM-DD"
    #[arg(short = 'B', long, default_value = NOT_BEFORE.as_str())]
    pub not_before: String,
    /// Validity period end date, in the format "YYYY-MM-DD"
    #[arg(short = 'A', long, default_value = NOT_AFTER.as_str())]
    pub not_after: String,
    /// Common name for the certificate
    #[arg(short = 'C', long, default_value = "Example CA")]
    pub common_name: String,
    /// Common name for the certificate
    #[arg(short = 'O', long, default_value = "Rustls Server Acceptor")]
    pub organization_name: String,
    /// Output directory for the certificate and key files
    #[arg(short, long, default_value = ".")]
    pub output_folder: PathBuf,
}

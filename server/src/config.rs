use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

static IP_V4_REGEX: Lazy<Regex> = Lazy::new(|| {
	Regex::new(r"^(?:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|localhost)$").unwrap()
});

/// Configuration for the server
#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct Config {
	/// The port to listen on
	#[validate(range(min = 1, max = 65535), custom(function = "validate_port"))]
	pub port: u16,

	/// The address to bind to
	#[validate(regex(
		path = * IP_V4_REGEX, message = "Host must be a valid IPv4 address or localhost, ':params.value' provided"
	))]
	pub host: String,
}

impl Config {
	/// Load the configuration from a file
	pub fn load(path: &PathBuf) -> anyhow::Result<Arc<Self>> {
		let path = std::env::current_dir().unwrap().join(path);
		if !path.exists() {
			return Err(anyhow::anyhow!(format!("Configuration file not found at {}", path.display())));
		}

		let file = std::fs::File::open(path)?;
		let config: Config = serde_json::from_reader(file)?;
		config.validate()?;

		Ok(Arc::new(config))
	}
}

/// Validate that the port is within the valid range
fn validate_port(port: u16) -> Result<(), ValidationError> {
	if port < 1024 && !nix::unistd::Uid::effective().is_root() {
		Err(
			ValidationError::new("__internal__")
				.with_message(Cow::from("Ports below 1024 require root privileges to bind"))
		)
	} else {
		Ok(())
	}
}

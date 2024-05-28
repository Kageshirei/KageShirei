use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

pub static IP_V4_REGEX: Lazy<Regex> =
	Lazy::new(|| Regex::new(r"^(?:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|localhost)$").unwrap());

/// Validate that the port is within the valid range
pub fn validate_port(port: u16) -> Result<(), ValidationError> {
	if port < 1024 && !nix::unistd::Uid::effective().is_root() {
		Err(ValidationError::new("__internal__").with_message(Cow::from(
			"Ports below 1024 require root privileges to bind",
		)))
	} else {
		Ok(())
	}
}

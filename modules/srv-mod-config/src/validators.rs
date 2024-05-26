use std::borrow::Cow;

use validator::ValidationError;

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

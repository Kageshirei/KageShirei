use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::config::validators;

static IP_V4_REGEX: Lazy<Regex> = Lazy::new(|| {
	Regex::new(r"^(?:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|localhost)$").unwrap()
});

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct ApiServerConfig {
	/// The port to listen on
	#[validate(range(min = 1, max = 65535), custom(function = "validators::validate_port"))]
	pub port: u16,

	/// The address to bind to
	#[validate(regex(
		path = * IP_V4_REGEX, message = "Host must be a valid IPv4 address or localhost, ':params.value' provided"
	))]
	pub host: String,
}
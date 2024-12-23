use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::validators;

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct ApiServerConfig {
    /// The port to listen on
    #[validate(
        range(min = 1, max = 65535),
        custom(function = "validators::validate_port")
    )]
    pub port: u16,
    /// The address to bind to
    #[validate(regex(
		path = * validators::IP_V4_REGEX, message = "Host must be a valid IPv4 address or localhost, ':params.value' provided"
	))]
    pub host: String,
    /// TLS configuration
    #[validate(nested)]
    pub tls:  Option<TlsConfig>,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct TlsConfig {
    /// Whether to enable TLS support for the API server
    pub enabled: bool,
    /// The port to listen on for TLS connections
    #[validate(
        range(min = 1, max = 65535),
        custom(function = "validators::validate_port")
    )]
    pub port:    u16,
    /// The address to bind to for TLS connections (defaults to the host address)
    pub host:    Option<String>,
    /// The path to the certificate file in pem format
    pub cert:    PathBuf,
    /// The path to the private key file in pem format
    pub key:     PathBuf,
}

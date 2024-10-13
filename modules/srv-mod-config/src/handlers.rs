use std::fmt::Display;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{api_server::TlsConfig, validators};

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct HandlerConfig {
    /// Whether the handler is enabled
    pub enabled:   bool,
    /// The type of handler
    pub r#type:    HandlerType,
    /// The protocols supported by the handler
    pub protocols: Vec<Protocol>,
    /// The port to listen on
    #[validate(
        range(min = 1, max = 65535),
        custom(function = "validators::validate_port")
    )]
    pub port:      u16,
    /// The address to bind to
    #[validate(regex(
		path = * validators::IP_V4_REGEX, message = "Host must be a valid IPv4 address or localhost, ':params.value' provided"
	))]
    pub host:      String,
    /// TLS configuration
    #[validate(nested)]
    pub tls:       Option<TlsConfig>,
    #[validate(nested)]
    pub security:  HandlerSecurityConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum HandlerType {
    /// The handler is an HTTP handler
    #[serde(rename = "http")]
    #[default]
    Http,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum Protocol {
    /// The protocol used during the communication is JSON
    #[serde(rename = "json")]
    #[default]
    Json,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone, Default)]
pub struct HandlerSecurityConfig {
    pub encryption_scheme: EncryptionScheme,
    pub algorithm:         Option<EncryptionAlgorithm>,
    pub encoder:           Option<Encoder>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum EncryptionScheme {
    /// No encryption is used
    #[serde(rename = "plain")]
    #[default]
    Plain,
    /// The encryption scheme is symmetric
    #[serde(rename = "symmetric")]
    Symmetric,
    /// The encryption scheme is asymmetric
    #[serde(rename = "asymmetric")]
    Asymmetric,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum EncryptionAlgorithm {
    /// The encryption algorithm is xchacha20-poly1305
    #[serde(rename = "xchacha20-poly1305")]
    #[default]
    XChaCha20Poly1305,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum Encoder {
    /// The encoder is hex
    #[serde(rename = "hex")]
    #[default]
    Hex,
    /// The encoder is base32
    #[serde(rename = "base32")]
    Base32,
    /// The encoder is base64
    #[serde(rename = "base64")]
    Base64,
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoder::Hex => write!(f, "hex"),
            Encoder::Base32 => write!(f, "base32"),
            Encoder::Base64 => write!(f, "base64"),
        }
    }
}

use serde::{Deserialize, Serialize};

/// The JWT claims for the api server
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
	aud: String,         // Optional. Audience
	exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
	iat: usize,          // Optional. Issued at (as UTC timestamp)
	iss: String,         // Optional. Issuer
	nbf: usize,          // Optional. Not Before (as UTC timestamp)
	sub: String,         // Optional. Subject (whom token refers to)
}
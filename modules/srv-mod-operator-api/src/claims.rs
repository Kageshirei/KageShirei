use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};

use super::{errors::ApiServerError, jwt_keys::API_SERVER_JWT_KEYS};

/// The JWT claims for the api server
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[allow(
    clippy::module_name_repetitions,
    reason = "The repetition in the name ensures clarity by explicitly identifying this struct as the representation \
              of JWT claims."
)]
pub struct JwtClaims {
    /// Expiration time (as UTC timestamp)
    pub exp: u64,
    /// Issued at (as UTC timestamp)
    pub iat: u64,
    /// Issuer
    pub iss: String,
    /// Not Before (as UTC timestamp)
    pub nbf: u64,
    /// Subject (whom token refers to, aka user id)
    pub sub: String,
}

impl JwtClaims {
    /// Create a new JWT claims object
    ///
    /// # Arguments
    ///
    /// * `sub` - The subject (whom token refers to, aka user id)
    pub fn new(sub: String, lifetime: chrono::Duration) -> Self {
        let now = chrono::Utc::now();

        Self {
            exp: (now + lifetime).timestamp() as u64,
            iat: now.timestamp() as u64,
            iss: "kageshirei-api-server".to_string(),
            nbf: now.timestamp() as u64,
            sub,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for JwtClaims
where
    S: Send + Sync,
{
    type Rejection = ApiServerError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| ApiServerError::InvalidToken)?;

        // extract the header from the token
        let header = jsonwebtoken::decode_header(bearer.token()).map_err(|_| ApiServerError::InvalidToken)?;

        // Ensure the token is signed with HS512
        if header.alg != jsonwebtoken::Algorithm::HS512 {
            return Err(ApiServerError::InvalidToken);
        }

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
        validation.set_issuer(&["kageshirei-api-server"]);
        validation.set_required_spec_claims(&["exp", "sub"]);
        validation.leeway = 30; // 30 seconds leeway for clock skew

        // Decode the user data
        let token_data = jsonwebtoken::decode::<JwtClaims>(
            bearer.token(),
            &API_SERVER_JWT_KEYS.get().unwrap().decoding,
            &validation,
        )
        .map_err(|_| ApiServerError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

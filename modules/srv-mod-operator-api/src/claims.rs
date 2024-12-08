//! The claims module for the API server

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, RequestPartsExt as _};
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
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "This is an implicit call to the implementation of the su into the DateTime type"
            )]
            exp: (now + lifetime).timestamp() as u64,
            iat: now.timestamp() as u64,
            iss: "kageshirei-api-server".to_owned(),
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
            .map_err(|_silenced| ApiServerError::InvalidToken)?;

        // extract the header from the token
        let header = jsonwebtoken::decode_header(bearer.token()).map_err(|_silenced| ApiServerError::InvalidToken)?;

        // Ensure the token is signed with HS512
        if header.alg != jsonwebtoken::Algorithm::HS512 {
            return Err(ApiServerError::InvalidToken);
        }

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
        validation.set_issuer(&["kageshirei-api-server"]);
        validation.set_required_spec_claims(&["exp", "sub"]);
        validation.leeway = 30; // 30 seconds leeway for clock skew

        // Decode the user data
        let token_data = jsonwebtoken::decode::<Self>(
            bearer.token(),
            &API_SERVER_JWT_KEYS
                .get()
                .ok_or(ApiServerError::InternalServerError)?
                .decoding,
            &validation,
        )
        .map_err(|_silenced| ApiServerError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        extract::FromRequestParts,
        http::{request::Parts, HeaderMap, HeaderValue, Request},
    };
    use jsonwebtoken::Header;
    use once_cell::sync::OnceCell;

    use super::*;
    use crate::jwt_keys::Keys;

    /// Initialize mock JWT keys
    fn init_keys() {
        let encoding = jsonwebtoken::EncodingKey::from_secret(b"my_secret_key");
        let decoding = jsonwebtoken::DecodingKey::from_secret(b"my_secret_key");

        // Set the JWT keys but don't panic if they already exist
        let _ = API_SERVER_JWT_KEYS.set(Keys {
            decoding,
            encoding,
        });
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_jwt_claims_new() {
        init_keys();

        let sub = "test-user-id".to_string();
        let lifetime = chrono::Duration::minutes(15);
        let claims = JwtClaims::new(sub.clone(), lifetime);

        assert_eq!(claims.sub, sub);
        assert_eq!(claims.iss, "kageshirei-api-server");
        assert!(claims.exp > claims.iat);
        assert!(claims.nbf <= claims.iat);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_from_request_parts_valid_token() {
        init_keys();

        // Create a valid JWT token
        let token = jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS512),
            &JwtClaims::new("test-user-id".to_string(), chrono::Duration::minutes(15)),
            &API_SERVER_JWT_KEYS.get().unwrap().encoding,
        )
        .unwrap();

        // Create an HTTP request and inject the Authorization header
        let mut request = Request::new(());
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let mut parts = request.into_parts().0; // Extract `Parts`

        let claims = JwtClaims::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(claims.sub, "test-user-id");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_from_request_parts_invalid_algorithm() {
        init_keys();

        // Create a token with an invalid algorithm (HS256 instead of HS512)
        let token = jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &JwtClaims::new("test-user-id".to_string(), chrono::Duration::minutes(15)),
            &API_SERVER_JWT_KEYS.get().unwrap().encoding,
        )
        .unwrap();

        // Create an HTTP request and inject the Authorization header
        let mut request = Request::new(());
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let mut parts = request.into_parts().0;

        let result = JwtClaims::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ApiServerError::InvalidToken);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_from_request_parts_missing_header() {
        init_keys();

        // Create an HTTP request with no Authorization header
        let request = Request::new(());
        let mut parts = request.into_parts().0;

        let result = JwtClaims::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), ApiServerError::InvalidToken);
    }
}

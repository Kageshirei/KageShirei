//! The public authentication route for the API server

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use srv_mod_entity::{
    active_enums::LogLevel,
    entities::{logs, user},
    sea_orm::{prelude::*, ActiveValue::Set},
};
use tracing::{info, instrument};

use crate::{
    claims::JwtClaims,
    errors::ApiServerError,
    jwt_keys::API_SERVER_JWT_KEYS,
    request_body_from_content_type::InferBody,
    state::ApiServerSharedState,
};

/// The payload for the public authentication route
#[derive(Debug, Deserialize, Serialize)]
struct PostPayload {
    /// The username for the user
    pub username: String,
    /// The password for the user
    pub password: String,
}

/// The response for the public authentication route
#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    /// The access token type
    pub access_token: String,
    /// The time in seconds until the JWT token expires
    pub expires_in:   u64,
    /// The JWT token
    pub token:        String,
}

/// The handler for the public authentication route
#[instrument(name = "POST /authenticate", skip_all)]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    InferBody(payload): InferBody<PostPayload>,
) -> Result<Json<PostResponse>, ApiServerError> {
    // Ensure the username and password are not empty
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(ApiServerError::MissingCredentials);
    }

    let db = state.db_pool.clone();

    // Fetch the user from the database
    let usr = user::Entity::find()
        .filter(user::Column::Username.eq(&payload.username))
        .one(&db)
        .await
        .map_err(|_silenced| ApiServerError::WrongCredentials)?
        .ok_or(ApiServerError::WrongCredentials)?;

    // Verify the password
    if !kageshirei_crypt::hash::argon::Hash::verify_password(&payload.password, &usr.password) {
        return Err(ApiServerError::WrongCredentials);
    }

    // Create the JWT token
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    let token_lifetime = chrono::Duration::minutes(15);
    let claims = JwtClaims::new(usr.id.clone(), token_lifetime);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &API_SERVER_JWT_KEYS.get().unwrap().encoding,
    )
    .map_err(|_silenced| ApiServerError::TokenCreation)?;

    // Log the authentication on the cli and db
    info!("User {} authenticated", usr.username);
    logs::ActiveModel {
        level: Set(LogLevel::Info),
        message: Set(Some(format!("User {} authenticated", usr.username))),
        title: Set("User Authenticated".to_owned()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(|_e| ApiServerError::InternalServerError)?;

    Ok(Json(PostResponse {
        access_token: "bearer".to_owned(),
        expires_in: token_lifetime.num_seconds() as u64,
        token,
    }))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/authenticate", post(post_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use serde::Deserialize;
    use srv_mod_entity::{
        entities::{agent, terminal_history},
        sea_orm::{Database, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::{jwt_keys::Keys, state::ApiServerState};

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                terminal_history::Entity::delete_many()
                    .exec(txn)
                    .await
                    .unwrap();
                user::Entity::delete_many().exec(txn).await.unwrap();

                Ok(())
            })
        })
        .await
        .unwrap();
    }

    async fn init() -> DatabaseConnection {
        let db_pool = Database::connect("postgresql://kageshirei:kageshirei@localhost/kageshirei")
            .await
            .unwrap();

        cleanup(db_pool.clone()).await;

        user::Entity::insert(user::ActiveModel {
            id:         Set("user_id".to_string()),
            username:   Set("valid_user".to_string()),
            password:   Set(kageshirei_crypt::hash::argon::Hash::make_password("valid_password").unwrap()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .exec(&db_pool)
        .await
        .unwrap();

        // First initialization should succeed
        let secret: &[u8] = b"my_secret_key";
        let keys = Keys::new(secret);
        if API_SERVER_JWT_KEYS.get().is_none() {
            assert!(API_SERVER_JWT_KEYS.set(keys).is_ok());
        }

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_successful_authentication() {
        // Mock database setup with specific agent records
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        let state = Arc::new(ApiServerState {
            config:           Arc::new(Default::default()),
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Mock the payload
        let payload = PostPayload {
            username: "valid_user".to_string(),
            password: "valid_password".to_string(),
        };

        let request = Request::builder()
            .method("POST")
            .uri("/authenticate")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        // Mock the InferBody extractor for the payload
        let infer_body = InferBody(payload);
        let result = post_handler(State(state), infer_body).await;

        // Verify the response
        assert!(result.is_ok());
        let response = result.unwrap();
        let post_response = response.0;

        assert_eq!(post_response.access_token, "bearer");
        assert_eq!(post_response.token.len(), 255); // check JWT token length
        assert_eq!(post_response.expires_in, 900); // 15 minutes in seconds
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_invalid_credentials_user_not_found() {
        // Mock database setup with specific agent records
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        let state = Arc::new(ApiServerState {
            config:           Arc::new(Default::default()),
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Mock the payload
        let payload = PostPayload {
            username: "invalid_user".to_string(),
            password: "invalid_password".to_string(),
        };

        let request = Request::builder()
            .method("POST")
            .uri("/authenticate")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        // Mock the InferBody extractor for the payload
        let infer_body = InferBody(payload);
        let result = post_handler(State(state), infer_body).await;

        // Assert the result is an error (wrong credentials)
        assert!(result.is_err());
        let response = result.err().unwrap();
        assert_eq!(response, ApiServerError::WrongCredentials);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_missing_credentials() {
        // Mock database setup with specific agent records
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        let state = Arc::new(ApiServerState {
            config:           Arc::new(Default::default()),
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Mock the payload
        let payload = PostPayload {
            username: "".to_string(),
            password: "".to_string(),
        };

        let infer_body = InferBody(payload);
        let result = post_handler(State(state), infer_body).await;

        // Assert the result is an error (missing credentials)
        assert!(result.is_err());
        let response = result.err().unwrap();
        assert_eq!(response, ApiServerError::MissingCredentials);
    }
}

//! Refresh token route module

use axum::{extract::State, routing::post, Json, Router};
use srv_mod_entity::{entities::user, sea_orm::prelude::*};
use tracing::{info, instrument};

use crate::{
    claims::JwtClaims,
    errors::ApiServerError,
    jwt_keys::API_SERVER_JWT_KEYS,
    routes::public::authenticate::PostResponse,
    state::ApiServerSharedState,
};

/// The handler for the public authentication route
#[instrument(name = "POST /refresh-token", skip(state))]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
) -> Result<Json<PostResponse>, ApiServerError> {
    let db = state.db_pool.clone();

    // Fetch the user from the database
    let current_user = user::Entity::find()
        .filter(user::Column::Id.eq(jwt_claims.sub))
        .one(&db)
        .await
        .map_err(|_silenced| ApiServerError::InvalidToken)?
        .ok_or(ApiServerError::InvalidToken)?;

    // Create the JWT token
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    let token_lifetime = chrono::Duration::minutes(15);
    let claims = JwtClaims::new(current_user.id, token_lifetime);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &API_SERVER_JWT_KEYS.get().unwrap().encoding,
    )
    .map_err(|_silenced| ApiServerError::TokenCreation)?;

    info!("User {} refreshed token", current_user.username);

    Ok(Json(PostResponse {
        access_token: "bearer".to_owned(),
        expires_in: token_lifetime.num_seconds() as u64,
        token,
    }))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/refresh-token", post(post_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use srv_mod_entity::{
        entities::{logs, read_logs, terminal_history},
        sea_orm::{ActiveValue::Set, Database, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::{
        errors::ApiServerError,
        jwt_keys::{Keys, API_SERVER_JWT_KEYS},
        routes::public::authenticate::PostResponse,
        state::ApiServerState,
    };

    // Initialize the database connection, setup, and cleanup
    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                terminal_history::Entity::delete_many()
                    .exec(txn)
                    .await
                    .unwrap();
                user::Entity::delete_many().exec(txn).await.unwrap();
                read_logs::Entity::delete_many().exec(txn).await.unwrap();
                logs::Entity::delete_many().exec(txn).await.unwrap();
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
            id:         Set("test_user".to_string()),
            username:   Set("testuser".to_string()),
            password:   Set("hashed_password".to_string()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    async fn test_post_handler_refresh_token() {
        let db = init().await;
        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Setup state with a mock API server state
        let state = Arc::new(ApiServerState {
            config:           Arc::new(Default::default()),
            db_pool:          db.clone(),
            broadcast_sender: sender,
        });

        // Setup the JWT key
        let secret: &[u8] = b"my_secret_key";
        let keys = Keys::new(secret);
        if API_SERVER_JWT_KEYS.get().is_none() {
            API_SERVER_JWT_KEYS.set(keys).ok().unwrap();
        }

        // Step 3: Create a valid JwtClaims object
        let jwt_claims = JwtClaims {
            sub: "test_user".to_string(),
            exp: (Utc::now().timestamp() + 3600) as u64,
            iat: 0,
            iss: "".to_string(),
            nbf: 0,
        };

        // Step 4: Call the post_handler directly
        let result = post_handler(State(state), jwt_claims).await;

        // Step 5: Assert the results
        assert!(result.is_ok());
        let response: PostResponse = result.unwrap().0;

        // Validate the token structure and content
        assert!(response.token.starts_with("eyJ")); // Check if it's a valid JWT
        assert_eq!(response.expires_in, 15 * 60); // Token lifetime in seconds
        assert_eq!(response.access_token, "bearer"); // Access token type
    }
}

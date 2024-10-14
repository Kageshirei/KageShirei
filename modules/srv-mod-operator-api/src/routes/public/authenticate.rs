use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
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

#[derive(Debug, Deserialize)]
struct AuthenticatePostPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatePostResponse {
    pub access_token: String,
    pub expires_in:   u64,
    pub token:        String,
}

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "POST /authenticate", skip_all)]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    InferBody(payload): InferBody<AuthenticatePostPayload>,
) -> Result<Json<AuthenticatePostResponse>, ApiServerError> {
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
        .map_err(|_| ApiServerError::WrongCredentials)?
        .ok_or(ApiServerError::WrongCredentials)?;

    // Verify the password
    if !kageshirei_crypt::argon::Argon2::verify_password(&payload.password, &usr.password) {
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
    .map_err(|_| ApiServerError::TokenCreation)?;

    // Log the authentication on the cli and db
    info!("User {} authenticated", usr.username);
    logs::ActiveModel {
        level: Set(LogLevel::Info),
        message: Set(Some(format!("User {} authenticated", usr.username))),
        title: Set("User Authenticated".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(|e| ApiServerError::InternalServerError)?;

    Ok(Json(AuthenticatePostResponse {
        access_token: "bearer".to_string(),
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
    use std::{path::PathBuf, sync::Arc};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        response::Response,
    };
    use serde_json::json;
    use srv_mod_config::{database::DatabaseConfig, RootConfig, SharedConfig};
    use srv_mod_database::{
        bb8,
        diesel,
        diesel::{Connection, PgConnection},
        diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl},
        diesel_migrations::MigrationHarness,
        migration::MIGRATIONS,
        models::user::CreateUser,
        schema::users,
        Pool,
    };
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    use super::*;
    use crate::{jwt_keys::Keys, state::ApiServerState};

    fn make_shared_config() -> SharedConfig {
        let config = RootConfig {
            database: DatabaseConfig {
                url: "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string(),
                ..DatabaseConfig::default()
            },
            ..RootConfig::default()
        };

        Arc::new(RwLock::new(config))
    }

    async fn generate_test_user(pool: Pool) {
        let mut connection = pool.get().await.unwrap();
        diesel::insert_into(users::table)
            .values(CreateUser::new("test".to_string(), "test".to_string()))
            .execute(&mut connection)
            .await
            .unwrap();
    }

    async fn drop_database(shared_config: SharedConfig) {
        let readonly_config = shared_config.read().await;
        let mut connection = PgConnection::establish(readonly_config.database.url.as_str()).unwrap();

        connection.revert_all_migrations(MIGRATIONS).unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }

    async fn make_pool(shared_config: SharedConfig) -> Pool {
        let readonly_config = shared_config.read().await;

        let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&readonly_config.database.url);
        Arc::new(
            bb8::Pool::builder()
                .max_size(1u32)
                .build(connection_manager)
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn test_authenticate_post() {
        API_SERVER_JWT_KEYS.get_or_init(|| {
            // This is a randomly generated key, it is not secure and should not be used in production,
            // copied from the sample configuration
            Keys::new(
                "TlwDBT0AKR+eRhG0s8nWCWZqggT3/ZNyFXZsOJBISH4u+t6Vs9wof7nAGzerhRmtm51u02rQ4yd3uIRDLxvwzw==".as_bytes(),
            )
        });

        let shared_config = make_shared_config();
        // Ensure the database is clean
        drop_database(shared_config.clone()).await;
        let pool = make_pool(shared_config.clone()).await;
        // Generate a test user
        generate_test_user(pool.clone()).await;

        let route_state = Arc::new(ApiServerState {
            config:  shared_config.clone(),
            db_pool: pool.clone(),
        });
        // init the app router
        let app = route(route_state.clone());

        // create the request
        let request_body = Body::from(
            json!({
                "username": "test",
                "password": "test"
            })
            .to_string(),
        );
        let request = Request::post("/authenticate")
            .header("Content-Type", "application/json")
            .body(request_body)
            .unwrap();

        // send the request
        let response = app
            .with_state(route_state.clone())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // unpack the response
        let body = serde_json::from_slice::<AuthenticatePostResponse>(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .as_ref(),
        )
        .unwrap();
        assert_eq!(body.access_token, "bearer");
        println!("Token: {}", body.token);

        // cleanup after test
        drop_database(shared_config.clone()).await;
    }
}

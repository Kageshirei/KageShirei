use axum::{debug_handler, Json, Router};
use axum::extract::State;
use axum::routing::post;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::async_main::api_server::claims::JwtClaims;
use crate::async_main::api_server::errors::ApiServerError;
use crate::async_main::api_server::jwt_keys::API_SERVER_JWT_KEYS;
use crate::async_main::api_server::request_body_from_content_type::InferBody;
use crate::async_main::api_server::state::ApiServerSharedState;
use crate::database::models::user::User;

#[derive(Debug, Deserialize)]
struct AuthenticatePostPayload {
	pub username: String,
	pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthenticatePostResponse {
	pub access_token: String,
	pub token: String,
}

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "POST /authenticate", skip(state))]
async fn post_handler(
	State(state): State<ApiServerSharedState>,
	InferBody(payload): InferBody<AuthenticatePostPayload>,
) -> Result<Json<AuthenticatePostResponse>, ApiServerError> {
	use crate::database::schema::users::dsl::*;

	// Ensure the username and password are not empty
	if payload.username.is_empty() || payload.password.is_empty() {
		return Err(ApiServerError::MissingCredentials);
	}

	let mut connection = state
		.db_pool
		.get()
		.await
		.map_err(|_| ApiServerError::InternalServerError)?;

	// Fetch the user from the database
	let user = users
		.filter(username.eq(&payload.username))
		.select(User::as_select())
		.first(&mut connection)
		.await
		.map_err(|_| ApiServerError::WrongCredentials)?;

	// Verify the password
	if !rs2_crypt::argon::Argon2::verify_password(&payload.password, &user.password) {
		return Err(ApiServerError::WrongCredentials);
	}

	// Create the JWT token
	let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
	let claims = JwtClaims::new(user.id.to_string());
	let token = jsonwebtoken::encode(
		&header,
		&claims,
		&API_SERVER_JWT_KEYS.get().unwrap().encoding,
	)
		.map_err(|_| ApiServerError::TokenCreation)?;

	Ok(Json(AuthenticatePostResponse {
		access_token: "bearer".to_string(),
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
	use std::path::PathBuf;
	use std::sync::Arc;

	use axum::body::{Body, to_bytes};
	use axum::http::{Request, StatusCode};
	use axum::response::Response;
	use diesel::{Connection, PgConnection};
	use diesel::associations::HasTable;
	use diesel_async::{AsyncPgConnection, RunQueryDsl};
	use diesel_async::pooled_connection::AsyncDieselConnectionManager;
	use diesel_migrations::MigrationHarness;
	use serde_json::json;
	use tokio::sync::RwLock;
	use tower::ServiceExt;

	use crate::async_main::api_server::jwt_keys::Keys;
	use crate::async_main::api_server::state::ApiServerState;
	use crate::cli::generate::operator::GenerateOperatorArguments;
	use crate::cli_cmd_generate::operator::generate_operator;
	use crate::config::config::{RootConfig, SharedConfig};
	use crate::config::database::DatabaseConfig;
	use crate::database::migration::MIGRATIONS;
	use crate::database::models::user::CreateUser;
	use crate::database::Pool;
	use crate::database::schema::users::dsl::users;
	use crate::database::schema::users::id;

	use super::*;

	fn make_shared_config() -> SharedConfig {
		let config = RootConfig {
			database: DatabaseConfig {
				url: "postgresql://rs2:rs2@localhost/rs2".to_string(),
				..DatabaseConfig::default()
			},
			..RootConfig::default()
		};

		Arc::new(RwLock::new(config))
	}

	async fn generate_test_user(pool: Pool) {
		let mut connection = pool.get().await.unwrap();
		diesel::insert_into(users::table())
			.values(CreateUser::new("test".to_string(), "test".to_string()))
			.execute(&mut connection)
			.await
			.unwrap();
	}

	async fn drop_database(shared_config: SharedConfig) {
		let readonly_config = shared_config.read().await;
		let mut connection =
			PgConnection::establish(readonly_config.database.url.as_str()).unwrap();

		connection.revert_all_migrations(MIGRATIONS).unwrap();
		connection.run_pending_migrations(MIGRATIONS).unwrap();
	}

	async fn make_pool(shared_config: SharedConfig) -> Pool {
		let readonly_config = shared_config.read().await;

		let connection_manager =
			AsyncDieselConnectionManager::<AsyncPgConnection>::new(&readonly_config.database.url);
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
			Keys::new("TlwDBT0AKR+eRhG0s8nWCWZqggT3/ZNyFXZsOJBISH4u+t6Vs9wof7nAGzerhRmtm51u02rQ4yd3uIRDLxvwzw==".as_bytes())
		});

		let shared_config = make_shared_config();
		// Ensure the database is clean
		drop_database(shared_config.clone()).await;
		let pool = make_pool(shared_config.clone()).await;
		// Generate a test user
		generate_test_user(pool.clone()).await;

		let route_state = Arc::new(ApiServerState {
			config: shared_config.clone(),
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

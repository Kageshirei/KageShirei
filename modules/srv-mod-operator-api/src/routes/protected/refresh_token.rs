use axum::{debug_handler, extract::State, routing::post, Json, Router};
use srv_mod_database::{
    diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
    diesel_async::RunQueryDsl,
    models::user::User,
};
use tracing::{info, instrument};

use crate::{
    claims::JwtClaims,
    errors::ApiServerError,
    jwt_keys::API_SERVER_JWT_KEYS,
    routes::public::authenticate::AuthenticatePostResponse,
    state::ApiServerSharedState,
};

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "POST /refresh-token", skip(state))]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
) -> Result<Json<AuthenticatePostResponse>, ApiServerError> {
    use srv_mod_database::schema::users::dsl::*;

    let mut connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| ApiServerError::InternalServerError)?;

    // Fetch the user from the database
    let user = users
        .filter(id.eq(&jwt_claims.sub))
        .select(User::as_select())
        .first(&mut connection)
        .await
        .map_err(|_| ApiServerError::InvalidToken)?;

    // Create the JWT token
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    let token_lifetime = chrono::Duration::minutes(15);
    let claims = JwtClaims::new(user.id.to_string(), token_lifetime);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &API_SERVER_JWT_KEYS.get().unwrap().encoding,
    )
        .map_err(|_| ApiServerError::TokenCreation)?;

    info!("User {} refreshed token", user.username);

    Ok(Json(AuthenticatePostResponse {
        access_token: "bearer".to_string(),
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

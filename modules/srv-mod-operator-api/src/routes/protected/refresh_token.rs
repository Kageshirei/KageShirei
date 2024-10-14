use axum::{debug_handler, extract::State, routing::post, Json, Router};
use srv_mod_entity::{entities::user, sea_orm::prelude::*};
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
    let db = state.db_pool.clone();

    // Fetch the user from the database
    let current_user = user::Entity::find()
        .filter(user::Column::Id.eq(jwt_claims.sub))
        .one(&db)
        .await
        .map_err(|_| ApiServerError::InvalidToken)?
        .ok_or(ApiServerError::InvalidToken)?;

    // Create the JWT token
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    let token_lifetime = chrono::Duration::minutes(15);
    let claims = JwtClaims::new(current_user.id.to_string(), token_lifetime);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &API_SERVER_JWT_KEYS.get().unwrap().encoding,
    )
    .map_err(|_| ApiServerError::TokenCreation)?;

    info!("User {} refreshed token", current_user.username);

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

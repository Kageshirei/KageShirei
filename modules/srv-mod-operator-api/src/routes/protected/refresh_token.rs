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

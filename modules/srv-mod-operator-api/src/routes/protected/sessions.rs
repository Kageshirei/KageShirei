use std::collections::HashMap;

use axum::{
    debug_handler,
    extract::{Query, State},
    routing::get,
    Json,
    Router,
};
use srv_mod_entity::{
    entities::agent,
    partial_models::agent::full_session_record::FullSessionRecord,
    sea_orm::{prelude::*, QueryOrder},
};
use tracing::{error, info, instrument};

use crate::{claims::JwtClaims, errors::ApiServerError, state::ApiServerSharedState};

/// The handler for the logs route
///
/// This handler fetches the logs from the database and returns them as a JSON response
#[debug_handler]
#[instrument(name = "GET /sessions", skip(state))]
async fn get_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
) -> Result<Json<Vec<FullSessionRecord>>, ApiServerError> {
    let db = state.db_pool.clone();

    // Fetch the user from the database
    let agents = agent::Entity::find()
        .order_by_desc(agent::Column::CreatedAt)
        .into_partial_model::<FullSessionRecord>()
        .all(&db)
        .await
        .map_err(|e| {
            error!("Failed to fetch agent sessions: {}", e.to_string());
            ApiServerError::InternalServerError
        })?;

    Ok(Json(agents))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/sessions", get(get_handler))
        .with_state(state)
}

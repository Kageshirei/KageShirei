use std::collections::HashMap;

use axum::{
    debug_handler,
    extract::{Query, State},
    routing::get,
    Json,
    Router,
};
use srv_mod_database::{
    diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
    diesel_async::RunQueryDsl,
    models::{
        agent::{Agent, FullSessionRecord},
        log::Log,
    },
    schema::{agents, logs},
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
    let mut connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| ApiServerError::InternalServerError)?;

    // Fetch the user from the database
    let mut agents = agents::table
        .order_by(agents::created_at.desc())
        .select(FullSessionRecord::as_select())
        .get_results::<FullSessionRecord>(&mut connection)
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

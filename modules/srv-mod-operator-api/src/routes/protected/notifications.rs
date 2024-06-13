use std::collections::HashMap;

use axum::{debug_handler, Json, Router};
use axum::extract::{Query, State};
use axum::routing::get;
use tracing::{error, info, instrument};

use srv_mod_database::diesel::{QueryDsl, SelectableHelper};
use srv_mod_database::diesel::ExpressionMethods;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::log::Log;
use srv_mod_database::models::notification::Notification;
use srv_mod_database::schema::{logs, notifications};

use crate::claims::JwtClaims;
use crate::errors::ApiServerError;
use crate::state::ApiServerSharedState;

/// The handler for the notifications route
///
/// This handler fetches the notifications from the database and returns them as a JSON response
///
/// # Request parameters
///
/// - `page` (optional): The page number to fetch. Defaults to 1
#[debug_handler]
#[instrument(name = "GET /notifications", skip(state))]
async fn get_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Notification>>, ApiServerError> {
    let mut connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| ApiServerError::InternalServerError)?;

    let page = params.get("page").and_then(|page| page.parse::<u32>().ok()).unwrap_or(1);
    let page_size = 50;

    // Fetch the user from the database
    let mut notification = notifications::table.order_by(notifications::created_at.desc())
        .offset(((page - 1) * page_size) as i64)
        .limit(page_size as i64)
        .select(Notification::as_select())
        .get_results::<Notification>(&mut connection)
        .await
        .map_err(|e| {
            error!("Failed to fetch notifications: {}", e.to_string());
            ApiServerError::InternalServerError
        })?;

    // Reverse the logs so the newest logs are at the bottom, this is required as the ordering of
    // elements must have most recent logs on top in order to split the logs into pages and
    // display them in the correct order
    notification.reverse();

    Ok(Json(notification))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/notifications", get(get_handler))
        .with_state(state)
}
//! Notifications route module

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
    Router,
};
use srv_mod_entity::{
    entities::{logs, read_logs},
    sea_orm::{prelude::*, Condition, QueryOrder as _},
};
use tracing::{error, instrument};

use crate::{claims::JwtClaims, errors::ApiServerError, state::ApiServerSharedState};

/// The handler for the notifications route
///
/// This handler fetches the notifications from the database and returns them as a JSON response
///
/// # Request parameters
///
/// - `page` (optional): The page number to fetch. Defaults to 1
#[instrument(name = "GET /notifications", skip(state))]
async fn get_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<logs::Model>>, ApiServerError> {
    let db = state.db_pool.clone();

    let mut page = params
        .get("page")
        .and_then(|page| page.parse::<i64>().ok())
        .unwrap_or(1);

    // Ensure the page is at least 1
    if page <= 0 {
        page = 1;
    }

    let page_size = 50;

    // Fetch the user from the database
    let notifications = logs::Entity::find()
        .filter(
            // Fetch only the logs that have not been read by the user yet
            // read_by = {user_id} AND logs.id = read_logs.log_id
            Condition::all()
                .add(read_logs::Column::ReadBy.ne(&jwt_claims.sub))
                .add(
                    Expr::col((
                        srv_mod_migration::m20241012_070519_create_logs_table::Logs::Table,
                        logs::Column::Id,
                    ))
                    .eq(Expr::col((
                        srv_mod_migration::m20241012_070519_create_logs_table::ReadLogs::Table,
                        read_logs::Column::LogId,
                    ))),
                ),
        )
        .order_by_asc(logs::Column::CreatedAt)
        .paginate(&db, page_size)
        .fetch_page(page.saturating_sub(1) as u64)
        .await
        .map_err(|e| {
            error!("Failed to fetch logs: {}", e.to_string());
            ApiServerError::InternalServerError
        })?;

    Ok(Json(notifications))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/notifications", get(get_handler))
        .with_state(state)
}

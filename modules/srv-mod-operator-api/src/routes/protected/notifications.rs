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
    sea_orm::{prelude::*, sea_query::IntoCondition as _, JoinType, QueryOrder as _, QuerySelect as _},
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

    // Fetch only the logs that have not been read by the user yet
    let notifications = logs::Entity::find()
        .join(
            JoinType::LeftJoin,
            logs::Relation::ReadLogs
                .def()
                .on_condition(move |_left, right| {
                    Expr::col((right, read_logs::Column::ReadBy))
                        .eq(jwt_claims.sub.clone())
                        .into_condition()
                }),
        )
        .filter(read_logs::Column::LogId.is_null())
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

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use axum::extract::{Query, State};
    use chrono::Utc;
    use srv_mod_entity::{
        active_enums::LogLevel,
        entities::{terminal_history, user},
        sea_orm::{ActiveValue::Set, Database, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::state::ApiServerState;

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
            id:         Set("user_id".to_string()),
            username:   Set("valid_user".to_string()),
            password:   Set(kageshirei_crypt::hash::argon::Hash::make_password("valid_password").unwrap()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .exec(&db_pool)
        .await
        .unwrap();

        logs::Entity::insert_many(vec![
            logs::ActiveModel {
                id:         Set("1".to_string()),
                level:      Set(LogLevel::Error),
                title:      Set("test log title".to_string()),
                message:    Set(Some("test log message".to_string())),
                extra:      Set(None),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
            },
            logs::ActiveModel {
                id:         Set("2".to_string()),
                level:      Set(LogLevel::Error),
                title:      Set("test log title".to_string()),
                message:    Set(Some("test log message".to_string())),
                extra:      Set(None),
                created_at: Set(Utc::now().naive_utc()),
                updated_at: Set(Utc::now().naive_utc()),
            },
        ])
        .exec(&db_pool)
        .await
        .unwrap();

        // Insert a read log entry for the user, ensuring this notification is unread
        read_logs::Entity::insert(read_logs::ActiveModel {
            log_id: Set("1".to_string()),
            read_by: Set("user_id".to_string()),
            read_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        })
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    // Create a mock handler test
    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_notifications_handler() {
        let db = init().await;
        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Setup state with a mock API server state
        let state = Arc::new(ApiServerState {
            config:           Arc::new(Default::default()),
            db_pool:          db.clone(),
            broadcast_sender: sender,
        });

        // Example of JWT claims (for testing purposes)
        let jwt_claims = JwtClaims {
            sub: "user_id".to_string(),
            exp: (Utc::now().timestamp() + 3600) as u64, // set expiry 1 hour from now
            iat: 0,
            iss: "".to_string(),
            nbf: 0,
        };

        // Setup request parameters (page number is optional)
        let params = HashMap::from([("page".to_string(), "1".to_string())]);

        // Simulate the get_handler call with a test request
        let query = Query(params);
        let result = get_handler(State(state), jwt_claims, query).await;

        // Verify the result
        assert!(result.is_ok());
        let notifications = result.unwrap().0;

        // Check that the result is a vector of logs (it should contain at least 1 notification)
        assert_eq!(notifications.len(), 1);
    }
}

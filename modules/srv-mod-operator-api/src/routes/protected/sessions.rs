//! Routes for fetching the available sessions

use axum::{extract::State, routing::get, Json, Router};
use srv_mod_entity::{
    entities::agent,
    partial_models::agent::full_session_record::FullSessionRecord,
    sea_orm::{prelude::*, QueryOrder as _},
};
use tracing::{error, instrument};

use crate::{claims::JwtClaims, errors::ApiServerError, state::ApiServerSharedState};

/// The handler for the logs route
///
/// This handler fetches the logs from the database and returns them as a JSON response
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use kageshirei_communication_protocol::{NetworkInterface, NetworkInterfaceArray};
    use srv_mod_entity::{
        active_enums::AgentIntegrity,
        entities::{logs, read_logs, terminal_history, user},
        sea_orm::{ActiveValue, ActiveValue::Set, Database, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::{
        claims::JwtClaims,
        errors::ApiServerError,
        state::{ApiServerSharedState, ApiServerState},
    };

    // Initialize the database connection, setup, and cleanup
    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                agent::Entity::delete_many().exec(txn).await.unwrap();

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

        let agent = agent::Entity::insert(agent::ActiveModel {
            id:                 Set("agent1".to_owned()),
            pid:                Set(1),
            secret:             Set("test".to_owned()),
            cwd:                Set("test".to_owned()),
            server_secret:      Set("test".to_owned()),
            operating_system:   Set("test".to_owned()),
            integrity:          Set(AgentIntegrity::Medium),
            updated_at:         Set(Utc::now().naive_utc()),
            domain:             Set(Some("test".to_owned())),
            hostname:           Set("test-hostname".to_owned()),
            network_interfaces: Set(NetworkInterfaceArray {
                network_interfaces: vec![NetworkInterface {
                    name:        Some("test".to_owned()),
                    dhcp_server: Some("test".to_owned()),
                    address:     Some("test".to_owned()),
                }],
            }),
            ppid:               Set(1),
            username:           Set("test".to_owned()),
            process_name:       Set("test".to_owned()),
            signature:          Set("test".to_owned()),
            terminated_at:      Set(None),
            created_at:         Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    async fn test_get_handler_sessions() {
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

        // Step 4: Call the get_handler directly
        let result = get_handler(State(state), jwt_claims).await;

        // Step 5: Assert the results
        assert!(result.is_ok());
        let sessions = result.unwrap().0;

        // Validate the fetched data
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "agent1");
    }
}

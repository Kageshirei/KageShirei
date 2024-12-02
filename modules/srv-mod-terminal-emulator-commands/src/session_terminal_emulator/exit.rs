//! Exit the terminal session on the client side

use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Handle the exit command
#[instrument]
pub async fn handle(_config: CommandHandlerArguments) -> Result<String, String> {
    debug!("Terminal command received");

    // Signal the frontend terminal emulator to exit the terminal session
    Ok("__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__".to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use srv_mod_entity::{
        entities::logs,
        sea_orm::{prelude::*, Database, DatabaseConnection, DbErr, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::command_handler::{
        CommandHandlerArguments,
        HandleArguments,
        HandleArgumentsSession,
        HandleArgumentsUser,
    };

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
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

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_exit() {
        // Mock database setup
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "test".to_owned(),
                hostname:   "test".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Call the handle function
        let result = handle(config).await;

        // Verify the result
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "__TERMINAL_EMULATOR_INTERNAL_HANDLE_EXIT__"
        );
    }
}

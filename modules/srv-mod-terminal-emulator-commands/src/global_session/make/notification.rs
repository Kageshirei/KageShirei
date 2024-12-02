//! This module contains the command handler for the `notification` command

use clap::Args;
use serde::Serialize;
use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_entity::{
    active_enums::LogLevel,
    entities::logs,
    sea_orm::{ActiveModelTrait as _, ActiveValue::Set},
};
use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalSessionMakeNotificationArguments {
    /// The level of the notification to send
    #[arg(short, long)]
    pub level:   LogLevel,
    /// The title of the notification
    #[arg(short, long)]
    pub title:   String,
    /// The message of the notification
    #[arg(short, long)]
    pub message: String,
}

/// Handle the history command
#[instrument]
pub async fn handle(
    config: CommandHandlerArguments,
    args: &TerminalSessionMakeNotificationArguments,
) -> Result<String, String> {
    debug!("Terminal command received");

    let db = config.db_pool.clone();

    let new_notification = logs::ActiveModel {
        level: Set(args.level.clone()),
        title: Set(args.title.clone()),
        message: Set(Some(args.message.clone())),
        ..Default::default()
    };
    let notification = new_notification
        .insert(&db)
        .await
        .map_err(|e| e.to_string())?;

    config
        .broadcast_sender
        .send(SseEvent {
            event: EventType::Log,
            id:    Some(notification.id.clone()),
            data:  serde_json::to_string(&notification).map_err(|e| e.to_string())?,
        })
        .map_err(|e| e.to_string())?;

    Ok("Notification sent".to_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use srv_mod_entity::sea_orm::{prelude::*, Database, DatabaseConnection, DbErr, TransactionTrait};
    use tokio::sync::broadcast;

    use super::*;
    use crate::command_handler::{HandleArguments, HandleArgumentsSession, HandleArgumentsUser};

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
    async fn test_handle_success() {
        use srv_mod_config::sse::common_server_state::EventType;
        use srv_mod_entity::active_enums::LogLevel;
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

        // Define arguments for the notification
        let args = TerminalSessionMakeNotificationArguments {
            level:   LogLevel::Info,
            title:   "Test Notification".to_owned(),
            message: "This is a test".to_owned(),
        };

        // Call the function
        let result = handle(config, &args).await;

        // Assert the function succeeded
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Notification sent");

        // Verify the broadcast message
        if let Ok(event) = receiver.recv().await {
            assert_eq!(event.event, EventType::Log);
            assert!(event.data.contains("Test Notification"));
            assert!(event.data.contains("This is a test"));
        }
        else {
            panic!("Broadcast message was not received");
        }
    }
}

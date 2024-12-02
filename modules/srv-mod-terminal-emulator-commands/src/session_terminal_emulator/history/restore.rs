//! Restore terminal session history commands

use clap::Args;
use serde::Serialize;
use serde_json::json;
use srv_mod_config::sse::common_server_state::{EventType, SseEvent};
use srv_mod_entity::{
    active_enums::LogLevel,
    entities::{logs, terminal_history},
    sea_orm::{prelude::*, ActiveValue::Set, Condition},
};
use tracing::{debug, instrument};

use crate::command_handler::CommandHandlerArguments;

/// Terminal session arguments for the global session terminal
#[derive(Args, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalSessionHistoryRestoreArguments {
    /// The list of command ids to restore
    pub command_ids: Vec<i64>,
}

fn make_sequence_counter_condition(ids: Vec<i64>) -> Condition {
    let mut condition = Condition::any();

    for id in ids.iter() {
        condition = condition.add(terminal_history::Column::SequenceCounter.eq(*id));
    }

    condition
}

/// Handle the clear command
#[instrument]
pub async fn handle(
    config: CommandHandlerArguments,
    args: &TerminalSessionHistoryRestoreArguments,
) -> Result<String, String> {
    debug!("Terminal command received");

    let db = config.db_pool.clone();

    // clear commands marking them as deleted (soft delete)
    let mut query = terminal_history::Entity::update_many();

    if config.session.session_id == "global" {
        query = query
            .filter(terminal_history::Column::IsGlobal.eq(true))
            .filter(terminal_history::Column::SessionId.is_null());
    }
    else {
        query = query.filter(terminal_history::Column::SessionId.eq(&config.session.session_id));
    }

    let result = query
        .filter(make_sequence_counter_condition(args.command_ids.clone()))
        .col_expr(
            terminal_history::Column::RestoredAt,
            Expr::value(chrono::Utc::now()),
        )
        .exec(&db)
        .await
        .map_err(|e| e.to_string())?;

    let message = format!("Restored {} command(s)", result.rows_affected);

    // create a log entry and save it
    let log = logs::ActiveModel {
        level: Set(LogLevel::Info),
        title: Set("Command(s) restored".to_owned()),
        message: Set(Some(message.clone())),
        extra: Set(Some(json!({
            "session": config.session.hostname,
            "ran_by": config.user.username,
        }))),
        ..Default::default()
    }
    .insert(&db)
    .await
    .map_err(|e| e.to_string())?;

    // broadcast the log
    config
        .broadcast_sender
        .send(SseEvent {
            data:  serde_json::to_string(&log).map_err(|e| e.to_string())?,
            event: EventType::Log,
            id:    Some(log.id),
        })
        .map_err(|e| e.to_string())?;

    // Signal the frontend terminal emulator to clear the terminal screen
    Ok(message)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use srv_mod_entity::{
        entities::user,
        sea_orm::{Database, DatabaseConnection, DbErr, TransactionTrait},
    };
    use tokio::sync::broadcast;

    use super::*;
    use crate::command_handler::{HandleArguments, HandleArgumentsSession, HandleArgumentsUser};

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                terminal_history::Entity::delete_many()
                    .exec(txn)
                    .await
                    .unwrap();

                user::Entity::delete_many().exec(txn).await.unwrap();

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

        let usr = user::Entity::insert(user::ActiveModel {
            id:         Set("test".to_owned()),
            username:   Set("test".to_owned()),
            password:   Set("test".to_owned()),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        terminal_history::Entity::insert_many(vec![
            terminal_history::ActiveModel {
                id:               Set("1".to_owned()),
                ran_by:           Set(usr.id.clone()),
                command:          Set("test".to_owned()),
                session_id:       Set(None),
                is_global:        Set(true),
                output:           Set(Some("test".to_owned())),
                exit_code:        Set(Some(1)),
                sequence_counter: Default::default(),
                deleted_at:       Set(Some(Utc::now().naive_utc())),
                restored_at:      Set(None),
                created_at:       Set(Utc::now().naive_utc()),
                updated_at:       Set(Utc::now().naive_utc()),
            },
            terminal_history::ActiveModel {
                id:               Set("2".to_owned()),
                ran_by:           Set(usr.id.clone()),
                command:          Set("test".to_owned()),
                session_id:       Set(None),
                is_global:        Set(true),
                output:           Set(Some("test".to_owned())),
                exit_code:        Set(Some(1)),
                sequence_counter: Default::default(),
                deleted_at:       Set(Some(Utc::now().naive_utc())),
                restored_at:      Set(None),
                created_at:       Set(Utc::now().naive_utc()),
                updated_at:       Set(Utc::now().naive_utc()),
            },
            terminal_history::ActiveModel {
                id:               Set("3".to_owned()),
                ran_by:           Set(usr.id.clone()),
                command:          Set("test".to_owned()),
                session_id:       Set(None),
                is_global:        Set(true),
                output:           Set(Some("test".to_owned())),
                exit_code:        Set(Some(1)),
                sequence_counter: Default::default(),
                deleted_at:       Set(Some(Utc::now().naive_utc())),
                restored_at:      Set(None),
                created_at:       Set(Utc::now().naive_utc()),
                updated_at:       Set(Utc::now().naive_utc()),
            },
        ])
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_handle_success() {
        // Mock database setup
        let db = init().await;

        // Mock broadcast channel
        let (sender, mut receiver) = broadcast::channel(1);

        // Create command handler arguments
        let config = Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: "global".to_owned(),
                hostname:   "kageshirei".to_owned(),
            },
            user:             HandleArgumentsUser {
                user_id:  "test".to_owned(),
                username: "test".to_owned(),
            },
            db_pool:          db,
            broadcast_sender: sender,
        });

        // Define input arguments
        let args = TerminalSessionHistoryRestoreArguments {
            command_ids: vec![1, 2, 3],
        };

        // Call the function
        let result = handle(config, &args).await;

        // Assert success
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Restored 3 command(s)");

        // Verify the broadcasted message
        if let Ok(event) = receiver.recv().await {
            assert_eq!(event.event, EventType::Log);
            assert!(event.data.contains("Restored 3 command(s)"));
        }
    }

    #[test]
    fn test_make_sequence_counter_condition() {
        use srv_mod_entity::{
            entities::terminal_history,
            sea_orm::{ColumnTrait, Condition},
        };

        // Define test input
        let ids = vec![1, 2, 3];

        // Generate condition
        let condition = make_sequence_counter_condition(ids);

        // Expected condition
        let expected_condition = Condition::any()
            .add(terminal_history::Column::SequenceCounter.eq(1i64))
            .add(terminal_history::Column::SequenceCounter.eq(2i64))
            .add(terminal_history::Column::SequenceCounter.eq(3i64));

        // Assert the conditions match
        assert_eq!(
            format!("{:?}", condition),
            format!("{:?}", expected_condition)
        );
    }
}

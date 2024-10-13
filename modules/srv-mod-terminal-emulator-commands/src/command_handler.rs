use std::{fmt::Debug, sync::Arc};

use srv_mod_config::sse::common_server_state::SseEvent;
use srv_mod_entity::sea_orm::DatabaseConnection;

pub trait CommandHandler: Debug {
    /// Handle the command
    fn handle_command(
        &self,
        config: CommandHandlerArguments,
    ) -> impl std::future::Future<Output = Result<String, String>> + Send;
}

pub type CommandHandlerArguments = Arc<HandleArguments>;

#[derive(Debug, Clone)]
pub struct HandleArguments {
    /// The session that ran the command
    pub session: HandleArgumentsSession,
    /// The user that ran the command
    pub user: HandleArgumentsUser,
    /// The database connection pool
    pub db_pool: DatabaseConnection,
    /// The broadcast sender for the API server
    pub broadcast_sender: tokio::sync::broadcast::Sender<SseEvent>,
}

#[derive(Debug, Clone)]
pub struct HandleArgumentsSession {
    pub session_id: String,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct HandleArgumentsUser {
    pub user_id: String,
    pub username: String,
}

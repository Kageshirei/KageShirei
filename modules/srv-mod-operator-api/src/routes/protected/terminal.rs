//! The terminal route module

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    response::Response,
    routing::post,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use srv_mod_entity::{
    entities::{agent, terminal_history, user},
    partial_models::terminal_history::full_history_record::FullHistoryRecord,
    sea_orm::{prelude::*, ActiveValue::Set, Condition, QueryOrder as _, QuerySelect as _},
};
use srv_mod_terminal_emulator_commands::{
    command_handler::{CommandHandler as _, HandleArguments, HandleArgumentsSession, HandleArgumentsUser},
    Command,
    StyledStr,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument};

use crate::{
    claims::JwtClaims,
    errors::ApiServerError,
    request_body_from_content_type::InferBody,
    state::ApiServerSharedState,
};

/// The payload for the terminal command route
#[derive(Deserialize, Serialize, Debug)]
struct TerminalCommand {
    /// The raw command written in the terminal emulator
    command:    String,
    /// The terminal session ID, if any. This is used to identify the terminal session (aka agent
    /// id). If empty the "global" terminal session is used.
    session_id: Option<String>,
}

/// The response for the terminal command route
#[derive(Debug, Serialize, Deserialize)]
struct TerminalCommandResponse {
    /// The terminal session ID, if any. This is used to identify the terminal session (aka agent
    /// id). If empty the "global" terminal session is used.
    session_id: Option<String>,
    /// The raw command written in the terminal emulator
    command:    String,
    /// The response from the terminal emulator
    response:   String,
}

/// Update the command state in the database
fn update_command_state(
    movable_response: String,
    storable_command_id: String,
    cloned_state: Arc<ApiServerSharedState>,
    exit_code: i32,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let movable_response = movable_response.as_str();
        let storable_command_id = storable_command_id.as_str();
        let db = cloned_state.db_pool.clone();

        loop {
            // Update the command in the database.
            // This update is fallible as a race condition exists where the command might not exist in the
            // database when the update is attempted.
            // If the update fails, sleep for 200ms before retrying.
            let result = terminal_history::Entity::update_many()
                .set(terminal_history::ActiveModel {
                    output: Set(Some(movable_response.to_owned())),
                    exit_code: Set(Some(exit_code)),
                    ..Default::default()
                })
                .filter(terminal_history::Column::Id.eq(storable_command_id))
                .exec(&db)
                .await;

            if let Ok(update_result) = result &&
                update_result.rows_affected > 0
            {
                break;
            }

            // Sleep for 200ms before retrying
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    })
}

/// Get the current username
///
/// # Arguments
///
/// - `db`: The database connection
/// - `user_id`: The user ID
/// - `session_id`: The session ID
/// - `command`: The command
///
/// # Returns
///
/// The username of the current user
async fn get_current_username(
    db: DatabaseConnection,
    user_id: &str,
    session_id: &str,
    command: &str,
) -> Result<String, Response> {
    let user = user::Entity::find()
        .filter(user::Column::Id.eq(user_id))
        .one(&db)
        .await
        .map_err(|e| ApiServerError::make_terminal_emulator_error(session_id, command, e.to_string().as_str()))?;

    if user.is_none() {
        return Err(ApiServerError::make_terminal_emulator_error(
            session_id,
            command,
            "User not found",
        ));
    }

    let user = user.unwrap();
    Ok(user.username)
}

/// Get the hostname of the current session
///
/// # Arguments
///
/// - `db`: The database connection
/// - `session_id`: The session ID
/// - `command`: The command
///
/// # Returns
///
/// The hostname of the current session
async fn get_hostname(db: DatabaseConnection, session_id: &str, command: &str) -> Result<String, Response> {
    if session_id == "global" {
        Ok("kageshirei".to_owned())
    }
    else {
        let agent = agent::Entity::find()
            .filter(agent::Column::Id.eq(session_id))
            .one(&db)
            .await
            .map_err(|e| ApiServerError::make_terminal_emulator_error(session_id, command, e.to_string().as_str()))?;

        if agent.is_none() {
            return Err(ApiServerError::make_terminal_emulator_error(
                session_id,
                command,
                "Agent not found",
            ));
        }

        let agent = agent.unwrap();
        Ok(agent.hostname)
    }
}

/// The handler for the public authentication route
#[instrument(name = "POST /terminal", skip(state))]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    InferBody(body): InferBody<TerminalCommand>,
) -> Result<Json<TerminalCommandResponse>, Response> {
    info!("Received terminal command");

    let mut pending_handlers = vec![];

    let state = Arc::new(state);

    // Ensure the session_id is not empty
    let session_id = body.session_id.unwrap_or("global".to_owned());

    // clone the session_id and command to be able to move them into the spawned thread
    let mut storable_command = terminal_history::ActiveModel {
        ran_by: Set(jwt_claims.sub.clone()),
        command: Set(body.command.clone()),
        ..Default::default()
    };

    if session_id != "global" {
        storable_command.session_id = Set(Some(session_id.clone()));
        storable_command.is_global = Set(false);
    }
    else {
        storable_command.session_id = Set(None);
        storable_command.is_global = Set(true);
    }

    // clone the id to be able to update the command once the output is ready
    let storable_command_id = storable_command.id.clone().unwrap();
    let cloned_state = state.clone();

    // Persist the command in the database, in a separate thread to avoid blocking the response
    pending_handlers.push(tokio::spawn(async move {
        let db = cloned_state.db_pool.clone();

        storable_command.insert(&db).await.unwrap();
    }));

    let cmd: Result<Box<Command>, StyledStr> = Command::from_raw(session_id.as_str(), body.command.as_str());

    debug!("Parsed command: {:?}", cmd);

    // If the command could not be parsed, return an error
    if let Err(e) = cmd {
        let response = e.ansi().to_string();
        let movable_response = response.clone();
        let cloned_state = state.clone();

        // Update the command in the database, in a separate thread to avoid blocking the response
        pending_handlers.push(update_command_state(
            movable_response,
            storable_command_id,
            cloned_state,
            1,
        ));

        // Wait for all the pending handlers to finish
        futures::future::join_all(pending_handlers).await;

        return Ok(Json(TerminalCommandResponse {
            session_id: Some(session_id),
            command: body.command,
            response,
        }));
    }

    let cmd = cmd.unwrap();

    // Get the hostname and username
    let (hostname, username) = tokio::join!(
        get_hostname(
            state.db_pool.clone(),
            session_id.as_str(),
            body.command.as_str()
        ),
        get_current_username(
            state.db_pool.clone(),
            jwt_claims.sub.as_str(),
            session_id.as_str(),
            body.command.as_str(),
        )
    );

    // Ensure both the hostname and username are available
    let (hostname, username) = match (hostname, username) {
        (Ok(hostname), Ok(username)) => (hostname, username),
        (Err(e), _) => return Err(e),
        (_, Err(e)) => return Err(e),
    };

    // Handle the command
    let response = cmd
        .handle_command(Arc::new(HandleArguments {
            session:          HandleArgumentsSession {
                session_id: session_id.clone(),
                hostname,
            },
            user:             HandleArgumentsUser {
                user_id: jwt_claims.sub,
                username,
            },
            db_pool:          state.db_pool.clone(),
            broadcast_sender: state.broadcast_sender.clone(),
        }))
        .await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            let response = e.clone();
            let movable_response = response.clone();
            let cloned_state = state.clone();

            // Update the command in the database, in a separate thread to avoid blocking the response
            pending_handlers.push(update_command_state(
                movable_response,
                storable_command_id,
                cloned_state,
                1,
            ));

            // Wait for all the pending handlers to finish
            futures::future::join_all(pending_handlers).await;

            return Err(ApiServerError::make_terminal_emulator_error(
                session_id.as_str(),
                body.command.as_str(),
                e.as_str(),
            ));
        },
    };

    let movable_response = response.clone();
    let cloned_state = state.clone();

    // Update the command in the database, in a separate thread to avoid blocking the response
    pending_handlers.push(update_command_state(
        movable_response,
        storable_command_id,
        cloned_state,
        0,
    ));

    // Wait for all the pending handlers to finish
    futures::future::join_all(pending_handlers).await;

    Ok(Json(TerminalCommandResponse {
        session_id: Some(session_id),
        command: serde_json::to_string(&cmd).unwrap(),
        response,
    }))
}

/// The handler for the notifications route
///
/// This handler fetches the notifications from the database and returns them as a JSON response
///
/// # Request parameters
///
/// - `page` (optional): The page number to fetch. Defaults to 1
#[instrument(name = "GET /terminal", skip(state))]
async fn get_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<FullHistoryRecord>>, ApiServerError> {
    let db = state.db_pool.clone();

    let fallback_session_id = "global".to_owned();
    let session_id_v = params.get("session_id").unwrap_or(&fallback_session_id);

    let mut page = params
        .get("page")
        .and_then(|page| page.parse::<i64>().ok())
        .unwrap_or(1);

    // Ensure the page is not less than 1
    if page <= 0 {
        page = 1;
    }

    let page_size = 50;

    // fetch the latest commands and their output from the database
    let retrieved_commands = terminal_history::Entity::find()
        .filter(
            Condition::all()
                .add(terminal_history::Column::SessionId.eq(session_id_v))
                .add(
                    Condition::any()
                        .add(terminal_history::Column::DeletedAt.is_null())
                        .add(
                            Condition::all()
                                .add(terminal_history::Column::RestoredAt.is_not_null())
                                .add(
                                    Expr::col(
                                        (
                                            srv_mod_migration::m20241012_070535_create_terminal_history_table::TerminalHistory::Table,
                                         terminal_history::Column::RestoredAt
                                        ),
                                    ).gt(
                                        Expr::col(
                                        (
                                            srv_mod_migration::m20241012_070535_create_terminal_history_table::TerminalHistory::Table,
                                         terminal_history::Column::DeletedAt
                                        ),
                                    )),
                                ),
                        ),
                ),
        )
        .order_by_asc(terminal_history::Column::CreatedAt)
        .into_partial_model::<FullHistoryRecord>()
        .paginate(&db, page_size)
        .fetch_page(page.saturating_sub(1) as u64)
        .await
        .map_err(|e| {
            error!("Failed to fetch commands: {}", e.to_string());
            ApiServerError::InternalServerError
        })?;

    Ok(Json(retrieved_commands))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/terminal", post(post_handler).get(get_handler))
        .with_state(state)
}

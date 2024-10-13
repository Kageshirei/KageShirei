use std::{collections::HashMap, path::PathBuf, sync::Arc};

use axum::{
    debug_handler,
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::post,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use srv_mod_database::{
    diesel,
    diesel::{
        associations::HasTable,
        BoolExpressionMethods,
        ExpressionMethods,
        Insertable,
        NullableExpressionMethods,
        QueryDsl,
        Queryable,
    },
    diesel_async::RunQueryDsl,
    models,
    models::{
        command::{CreateCommand, FullHistoryRecord},
        notification::Notification,
    },
    schema::{agents, commands, notifications, users},
};
use srv_mod_terminal_emulator_commands::{
    command_handler::{CommandHandler, HandleArguments, HandleArgumentsSession, HandleArgumentsUser},
    global_session::GlobalSessionTerminalEmulatorCommands,
    session_terminal_emulator::SessionTerminalEmulatorCommands,
    Command,
    StyledStr,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument};

use crate::{
    claims::JwtClaims,
    errors::ApiServerError,
    request_body_from_content_type::InferBody,
    routes::public::authenticate::AuthenticatePostResponse,
    state::ApiServerSharedState,
};

#[derive(Deserialize, Debug)]
struct TerminalCommand {
    /// The raw command written in the terminal emulator
    command: String,
    /// The terminal session ID, if any. This is used to identify the terminal session (aka agent id). If empty the
    /// "global" terminal session is used.
    session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TerminalCommandResponse {
    /// The terminal session ID, if any. This is used to identify the terminal session (aka agent id). If empty the
    /// "global" terminal session is used.
    session_id: Option<String>,
    /// The raw command written in the terminal emulator
    command: String,
    /// The response from the terminal emulator
    response: String,
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
        let mut connection = cloned_state
            .db_pool
            .get()
            .await
            .map_err(|_| ApiServerError::InternalServerError.into_response())
            .unwrap();

        loop {
            // Update the command in the database.
            // This update is fallible as a race condition exists where the command might not exist in the database
            // when the update is attempted.
            // If the update fails, sleep for 1 second before retrying.
            let result = diesel::update(commands::dsl::commands::table())
                .filter(commands::id.eq(storable_command_id))
                .set((
                    commands::dsl::output.eq(movable_response),
                    commands::dsl::exit_code.eq(exit_code),
                ))
                .execute(&mut connection)
                .await;

            if let Ok(affected_rows) = result &&
                affected_rows > 0
            {
                break;
            }

            // Sleep for a second before retrying
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    })
}

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "POST /terminal", skip(state))]
async fn post_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    InferBody(mut body): InferBody<TerminalCommand>,
) -> Result<Json<TerminalCommandResponse>, Response> {
    info!("Received terminal command");

    let mut pending_handlers = vec![];

    let state = Arc::new(state);

    // Ensure the session_id is not empty
    let session_id = body.session_id.unwrap_or("global".to_string());

    // clone the session_id and command to be able to move them into the spawned thread
    let movable_cmd = body.command.clone();
    let mut storable_command = CreateCommand::new(jwt_claims.sub.clone(), session_id.clone());

    // clone the id to be able to update the command once the output is ready
    let storable_command_id = storable_command.id.clone();
    let cloned_state = state.clone();

    // Persist the command in the database, in a separate thread to avoid blocking the response
    pending_handlers.push(tokio::spawn(async move {
        let mut connection = cloned_state
            .db_pool
            .get()
            .await
            .map_err(|_| ApiServerError::InternalServerError.into_response())
            .unwrap();

        storable_command.command = movable_cmd;
        storable_command
            .insert_into(commands::dsl::commands::table())
            .execute(&mut connection)
            .await
            .unwrap();
    }));

    let cmd: Result<Box<Command>, StyledStr> = Command::from_raw(session_id.as_str(), body.command.as_str());

    debug!("Parsed command: {:?}", cmd);

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
            session_id: Some(session_id.to_string()),
            command: body.command,
            response,
        }));
    }

    let cmd = cmd.unwrap();

    let hostname = if session_id == "global" {
        "RS2".to_string()
    } else {
        let mut connection = state.db_pool.get().await.unwrap();
        agents::table
            .select((agents::hostname))
            .filter(agents::id.eq(session_id.as_str()))
            .first::<String>(&mut connection)
            .await
            .map_err(|e| {
                ApiServerError::make_terminal_emulator_error(
                    session_id.as_str(),
                    body.command.as_str(),
                    e.to_string().as_str(),
                )
            })?
    };

    let username = {
        let mut connection = state.db_pool.get().await.unwrap();
        users::table
            .select(users::username)
            .filter(users::id.eq(jwt_claims.sub.as_str()))
            .first::<String>(&mut connection)
            .await
            .map_err(|e| {
                ApiServerError::make_terminal_emulator_error(
                    session_id.as_str(),
                    body.command.as_str(),
                    e.to_string().as_str(),
                )
            })?
    };

    // Handle the command
    let response = cmd
        .handle_command(Arc::new(HandleArguments {
            session: HandleArgumentsSession {
                session_id: session_id.clone(),
                hostname,
            },
            user: HandleArgumentsUser {
                user_id: jwt_claims.sub,
                username,
            },
            db_pool: state.db_pool.clone(),
            broadcast_sender: state.broadcast_sender.clone(),
        }))
        .await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            let response = e.to_string();
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
                e.to_string().as_str(),
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
        session_id: Some(session_id.to_string()),
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
#[debug_handler]
#[instrument(name = "GET /terminal", skip(state))]
async fn get_handler(
    State(state): State<ApiServerSharedState>,
    jwt_claims: JwtClaims,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<FullHistoryRecord>>, ApiServerError> {
    let mut connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| ApiServerError::InternalServerError)?;

    let fallback_session_id = "global".to_string();
    let session_id_v = params.get("session_id").unwrap_or(&fallback_session_id);

    let page = params
        .get("page")
        .and_then(|page| page.parse::<u32>().ok())
        .unwrap_or(1);
    let page_size = 50;

    // fetch the latest commands and their output from the database
    let mut retrieved_commands = commands::table
        .inner_join(users::table)
        .select((
            commands::sequence_counter.nullable(),
            commands::command,
            commands::output.nullable(),
            commands::exit_code.nullable(),
            users::username,
            commands::created_at,
        ))
        .filter(commands::session_id.eq(session_id_v))
        .filter(
            // Select only commands that are not deleted or have been restored after deletion
            // deleted_at == null || (restored_at != null && restored_at > deleted_at)
            commands::deleted_at.is_null().or(commands::restored_at
                .is_not_null()
                .and(commands::restored_at.gt(commands::deleted_at))),
        )
        .order_by(commands::created_at.desc())
        .offset(((page - 1) * page_size) as i64)
        .limit(page_size as i64)
        .get_results::<FullHistoryRecord>(&mut connection)
        .await
        .map_err(|e| {
            error!("Failed to fetch commands: {}", e.to_string());
            ApiServerError::InternalServerError
        })?;

    // Reverse the logs so the newest logs are at the bottom, this is required as the ordering of
    // elements must have most recent logs on top in order to split the logs into pages and
    // display them in the correct order
    retrieved_commands.reverse();

    Ok(Json(retrieved_commands))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
    Router::new()
        .route("/terminal", post(post_handler).get(get_handler))
        .with_state(state)
}

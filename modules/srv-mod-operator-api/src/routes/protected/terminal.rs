use std::path::PathBuf;
use std::sync::Arc;

use axum::{debug_handler, Json, Router};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument};

use srv_mod_database::{diesel, models};
use srv_mod_database::diesel::associations::HasTable;
use srv_mod_database::diesel::ExpressionMethods;
use srv_mod_database::diesel::Insertable;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::command::CreateCommand;
use srv_mod_database::schema::commands;
use srv_mod_terminal_emulator_commands::{Command, StyledStr};
use srv_mod_terminal_emulator_commands::command_handler::{CommandHandler, SerializableCommandHandler};
use srv_mod_terminal_emulator_commands::global_session::GlobalSessionTerminalEmulatorCommands;
use srv_mod_terminal_emulator_commands::session_terminal_emulator::SessionTerminalEmulatorCommands;

use crate::claims::JwtClaims;
use crate::errors::ApiServerError;
use crate::request_body_from_content_type::InferBody;
use crate::routes::public::authenticate::AuthenticatePostResponse;
use crate::state::ApiServerSharedState;

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
) -> JoinHandle<()> {
	tokio::spawn(async move {
		let movable_response = movable_response.as_str();
		let storable_command_id = storable_command_id.as_str();
		let mut connection = cloned_state.db_pool
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
				.set((
					commands::dsl::output.eq(movable_response),
					commands::dsl::exit_code.eq(1),
				))
				.filter(commands::id.eq(storable_command_id))
				.execute(&mut connection)
				.await;

			if result.is_ok() {
				break;
			}

			// Sleep for a second before retrying
			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
	pending_handlers.push(
		tokio::spawn(async move {
			let mut connection = cloned_state.db_pool
			                                 .get()
			                                 .await
			                                 .map_err(|_| ApiServerError::InternalServerError.into_response())
			                                 .unwrap();

			storable_command.command = movable_cmd;
			storable_command.insert_into(commands::dsl::commands::table())
			                .execute(&mut connection)
			                .await
			                .unwrap();
		})
	);

	let cmd: Result<Box<Command>, StyledStr> = Command::from_raw(session_id.as_str(), body.command.as_str());

	debug!("Parsed command: {:?}", cmd);

	if let Err(e) = cmd {
		let response = e.ansi().to_string();
		let movable_response = response.clone();
		let cloned_state = state.clone();

		// Update the command in the database, in a separate thread to avoid blocking the response
		pending_handlers.push(update_command_state(movable_response, storable_command_id, cloned_state));

		// Wait for all the pending handlers to finish
		futures::future::join_all(pending_handlers).await;

		return Ok(Json(TerminalCommandResponse {
			session_id: Some(session_id.to_string()),
			command: body.command,
			response,
		}));
	}

	let cmd = cmd.unwrap();

	// Handle the command
	let response = cmd.handle_command(session_id.as_str())
	                  .map_err(|e| ApiServerError::make_terminal_emulator_error(
		                  session_id.as_str(),
		                  body.command,
		                  e.to_string().as_str(),
	                  ))?;

	let movable_response = response.clone();
	let cloned_state = state.clone();

	// Update the command in the database, in a separate thread to avoid blocking the response
	pending_handlers.push(update_command_state(movable_response, storable_command_id, cloned_state));

	// Wait for all the pending handlers to finish
	futures::future::join_all(pending_handlers).await;

	Ok(Json(TerminalCommandResponse {
		session_id: Some(session_id.to_string()),
		command: serde_json::to_string(&cmd).unwrap(),
		response,
	}))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.route("/terminal", post(post_handler))
		.with_state(state)
}
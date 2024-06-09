use std::path::PathBuf;

use axum::{debug_handler, Json, Router};
use axum::extract::State;
use axum::response::Response;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use srv_mod_terminal_emulator_commands::TerminalEmulatorCommands;

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

/// The handler for the public authentication route
#[debug_handler]
#[instrument(name = "POST /terminal", skip(state))]
async fn post_handler(
	State(state): State<ApiServerSharedState>,
	jwt_claims: JwtClaims,
	InferBody(body): InferBody<TerminalCommand>,
) -> Result<Json<TerminalCommandResponse>, Response> {
	info!("Received terminal command");

	let cmd = TerminalEmulatorCommands::from_raw(body.command.clone());

	debug!("Parsed command: {:?}", cmd);

	if let Err(e) = cmd {
		return Ok(Json(TerminalCommandResponse {
			session_id: body.session_id,
			command: body.command,
			response: e.ansi().to_string(),
		}));
	}

	let cmd = cmd.unwrap();

	Ok(Json(TerminalCommandResponse {
		session_id: body.session_id.clone(),
		command: serde_json::to_string(&cmd).unwrap(),
		response: cmd.handle_command()
		             .map_err(|e| ApiServerError::make_terminal_emulator_error(
			             body.session_id,
			             body.command,
			             e.to_string().as_str(),
		             ))?,
	}))
}

/// Creates the public authentication routes
pub fn route(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.route("/terminal", post(post_handler))
		.with_state(state)
}
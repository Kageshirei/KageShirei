//! This module contains the handlers for the agent command response and retrieval with heuristic
//! handling capabilities

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use srv_mod_handler_base::{handle_command_result, handle_command_retrieval, state::HandlerSharedState};
use tracing::{instrument, warn};

use crate::parse_base_handler_response::parse_base_handler_response;

/// The handler for the agent command response
#[instrument(skip(state, body, headers))]
async fn handle_post_request(
    State(state): State<HandlerSharedState>,
    headers: HeaderMap,
    body: Bytes,
    id: String,
) -> Response<Body> {
    parse_base_handler_response(handle_command_result(state, body.to_vec(), headers, id).await)
}

/// The handler for the agent command retrieval
#[instrument(skip(state))]
async fn handle_get_request(State(state): State<HandlerSharedState>, id: String) -> Response<Body> {
    parse_base_handler_response(handle_command_retrieval(state, id).await)
}

/// This kind of route uses the first path parameter as the index of the id in the path
///
/// # Example 1
/// Path: `/3/this/is/a/dd1g8uw209me6bin2unm9u38mhmp23ic/sample/path` <br/>
/// `-----|N|0---|1-|2|3-------------------------------|4-----|5----`
///
/// ### Explanation
/// - N: id_position, defines the position of the id in the path, 3 in the example, can be any
///   number
/// - 0 to 2: decoy strings, unused
/// - 3: the actual id of the request to parse
/// - 4+: other decoy strings, unused
///
/// # Example 2
/// Path: `/2,3,5/this/is/dd1g8uw/209me6bin2unm/a/9u38mhmp23ic/sample/path` <br/>
/// `-----|N,N,N|0---|1-|2------|3------------|4|------------|5-----|6---`
///
/// WHERE "," can be any of the following:
/// - ","
/// - ";"
/// - ":"
/// - "."
/// - "-"
/// - "_"
/// - " "
/// - "|"
/// - "$"
///
/// ### Explanation
/// - N: id_position, defines the position of the id in the path, 2, 3, 5 in the example, can be any
///   number, with any of the allowed separators
/// - 0 and 1: decoy strings, unused
/// - 2 and 3: fragments of the id, to be concatenated
/// - 4: decoy string, unused
/// - 5: last fragment of the id, to be concatenated
/// - 6+: other decoy strings, unused
pub fn heuristic_variant_1(Path((id_position, path)): Path<(String, String)>) -> Option<String> {
    // Split the id_position string by allowed separators
    let separators = [',', ';', ':', '.', '-', '_', ' ', '|', '$'];
    let id_positions = id_position
        .split(|c| separators.contains(&c))
        .map(|s| s.parse::<usize>())
        .collect::<Result<Vec<usize>, _>>()
        .map_err(|_silenced| StatusCode::BAD_REQUEST);

    if id_positions.is_err() {
        // if the id_position is not a number, return a bad request
        warn!("Unknown format for heuristic handling of requests (v1), request refused");
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return None;
    }

    // Unwrap the id_positions
    let id_positions = id_positions.unwrap();

    // Split the path by '/'
    let parts: Vec<&str> = path.split('/').collect();

    // Concatenate the fragments of the ID, appending an empty string for undefined positions
    let id = id_positions
        .iter()
        .map(|&pos| parts.get(pos).unwrap_or(&"").as_str())
        .collect::<Vec<&str>>()
        .join("");

    if id.len() != 32 {
        return None;
    }

    Some(id)
}

/// This kind of route automatically takes the first string matching the ID length (32) as the
/// request ID
///
/// # Example
/// Path: `/this/is/a/dd1g8uw209me6bin2unm9u38mhmp23ic/sample/path` <br/>
/// `-----|0---|1-|2|3-------------------------------|4-----|5---`
///
/// ### Explanation
/// - 0 to 2: decoy strings, unused
/// - 3: the actual id of the request to parse
/// - 4+: other decoy strings, unused
pub fn heuristic_handler_variant_2(Path(path): Path<String>) -> Option<String> {
    // Extract the ID by finding the first 32-character segment
    let id = path
        .split('/')
        .find(|&part| part.len() == 32)
        .ok_or(StatusCode::BAD_REQUEST);

    if id.is_err() {
        // if the id is not found, return a bad request
        warn!("Unknown format for heuristic handling of requests (v2), request refused");
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return None;
    }

    // Unwrap the id
    Some(id.unwrap().to_owned())
}

/// This route handles a generic path and uses heuristics to determine the ID and handle the request
async fn unified_post_handler(
    path: Path<String>,
    state: State<HandlerSharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    let pieces = path.split('/').collect::<Vec<&str>>();

    if pieces.len() >= 2 &&
        let Some(id) = heuristic_variant_1(Path((
            pieces.get(0).unwrap_or(&"").to_string(),
            pieces.iter().skip(1).map(|v| *v).collect::<String>(),
        )))
    {
        return handle_post_request(state, headers, body, id).await;
    }

    let id = heuristic_handler_variant_2(path);

    if id.is_none() {
        warn!("Unknown format for heuristic handling of requests, request refused");

        // always return OK to avoid leaking information
        return (StatusCode::OK, "").into_response();
    }

    // handle the request
    let id = id.unwrap();
    handle_post_request(state, headers, body, id).await
}

/// This route handles a generic path and uses heuristics to determine the ID and handle the request
async fn unified_get_handler(path: Path<String>, state: State<HandlerSharedState>) -> Response<Body> {
    let pieces = path.split('/').collect::<Vec<&str>>();

    if pieces.len() >= 2 &&
        let Some(id) = heuristic_variant_1(Path((
            pieces.get(0).unwrap_or(&"").to_string(),
            pieces.iter().skip(1).map(|v| *v).collect::<String>(),
        )))
    {
        return handle_get_request(state, id).await;
    }

    let id = heuristic_handler_variant_2(path);

    if id.is_none() {
        warn!("Unknown format for heuristic handling of requests, request refused");

        // always return OK to avoid leaking information
        return (StatusCode::OK, "").into_response();
    }

    // handle the request
    let id = id.unwrap();
    handle_get_request(state, id).await
}

/// Creates the routes for the commands handlers
pub fn route(state: HandlerSharedState) -> Router<HandlerSharedState> {
    Router::new()
        .route(
            "/*path",
            post(unified_post_handler).get(unified_get_handler),
        )
        .with_state(state)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_heuristic_variant_1_valid_path() {
        use axum::extract::Path;

        // Input parameters
        let id_position = "2,3,5".to_string();
        let path = "this/is/dd1g8uw/209me6bin2unm/a/9u38mhmp23ic/sample/path".to_string();
        let path_params = Path((id_position, path));

        // Execute the function
        let result = heuristic_variant_1(path_params);

        // Expected ID concatenation from positions 2, 3, and 5
        let expected_id = "dd1g8uw209me6bin2unm9u38mhmp23ic";

        // Assert result
        assert_eq!(result.unwrap(), expected_id);
    }

    #[test]
    fn test_heuristic_variant_1_invalid_id_position() {
        use axum::extract::Path;

        // Invalid id_position (non-numeric)
        let id_position = "invalid".to_string();
        let path = "this/is/a/bad/path".to_string();
        let path_params = Path((id_position, path));

        // Execute the function
        let result = heuristic_variant_1(path_params);

        // Expect empty response due to invalid id_position
        assert!(result.is_none());
    }

    #[test]
    fn test_heuristic_variant_1_out_of_bounds_position() {
        use axum::extract::Path;

        // id_position refers to an out-of-bounds index
        let id_position = "100".to_string();
        let path = "this/is/a/bad/path".to_string();
        let path_params = Path((id_position, path));

        // Execute the function
        let result = heuristic_variant_1(path_params);

        // Expect empty string since the position is out of bounds
        assert!(result.is_none());
    }

    #[test]
    fn test_heuristic_handler_variant_2_valid_path() {
        use axum::extract::Path;

        // Input path with a valid 32-character ID
        let path = "this/is/a/dd1g8uw209me6bin2unm9u38mhmp23ic/sample/path".to_string();
        let path_param = Path(path);

        // Execute the function
        let result = heuristic_handler_variant_2(path_param);

        // Expected ID
        let expected_id = "dd1g8uw209me6bin2unm9u38mhmp23ic";

        // Assert result
        assert_eq!(result.unwrap(), expected_id);
    }

    #[test]
    fn test_heuristic_handler_variant_2_missing_id() {
        use axum::extract::Path;

        // Input path without a 32-character ID
        let path = "this/is/a/sample/path".to_string();
        let path_param = Path(path);

        // Execute the function
        let result = heuristic_handler_variant_2(path_param);

        // Expect empty string since no valid ID is found
        assert!(result.is_none());
    }

    #[test]
    fn test_heuristic_handler_variant_2_multiple_valid_ids() {
        use axum::extract::Path;

        // Input path with multiple valid 32-character IDs
        let path = "this/is/a/dd1g8uw209me6bin2unm9u38mhmp23ic/dd1g8uw209me6bin2unm9u38mhmp99ic/path".to_string();
        let path_param = Path(path);

        // Execute the function
        let result = heuristic_handler_variant_2(path_param);

        // Expect the first valid ID found
        let expected_id = "dd1g8uw209me6bin2unm9u38mhmp23ic";

        // Assert result
        assert_eq!(result.unwrap(), expected_id);
    }
}

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use kageshirei_crypt::encoder::{
    base32::Base32Encoder,
    base64::Base64Encoder,
    hex::HexEncoder,
    Encoder as CryptEncoder,
};
use srv_mod_config::handlers::{Encoder, EncryptionScheme};
use srv_mod_handler_base::{handle_command_result, state::HandlerSharedState};
use tracing::{instrument, warn};

/// The handler for the agent checking operation
#[instrument(skip(state, body, headers))]
async fn handle_request(
    State(state): State<HandlerSharedState>,
    headers: HeaderMap,
    body: Bytes,
    id: String,
) -> Response<Body> {
    handle_command_result(state, body, headers, id).await
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
pub async fn heuristic_handler_variant_1(
    Path((id_position, path)): Path<(String, String)>,
    State(state): State<HandlerSharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    // Split the id_position string by allowed separators
    let separators = [',', ';', ':', '.', '-', '_', ' ', '|', '$'];
    let id_positions = id_position
        .split(|c| separators.contains(&c))
        .map(|s| s.parse::<usize>())
        .collect::<Result<Vec<usize>, _>>()
        .map_err(|_| StatusCode::BAD_REQUEST);

    if id_positions.is_err() {
        // if the id_position is not a number, return a bad request
        warn!("Unknown format for heuristic handling of requests (v1), request refused");
        warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

        // always return OK to avoid leaking information
        return (StatusCode::OK, "").into_response();
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

    handle_request(axum::extract::State(state), headers, body, id).await
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
pub async fn heuristic_handler_variant_2(
    Path(path): Path<String>,
    State(state): State<HandlerSharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
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
        return (StatusCode::OK, "").into_response();
    }

    // Unwrap the id
    let id = id.unwrap().to_string();

    handle_request(axum::extract::State(state), headers, body, id).await
}

/// Creates the routes for the commands handlers
pub fn route(state: HandlerSharedState) -> Router<HandlerSharedState> {
    // TODO: Implement the command retrieval using the base handler, simple stub already present in
    // lib.rs
    Router::new()
        .route("/:id_position/*path", post(heuristic_handler_variant_1))
        .route("/*path", post(heuristic_handler_variant_2))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::Request;
    use bytes::{BufMut, BytesMut};
    use kageshirei_communication_protocol::{
        communication::checkin::{Checkin, PartialCheckin},
        magic_numbers,
    };
    use serial_test::serial;
    use srv_mod_config::{
        handlers,
        handlers::{HandlerConfig, HandlerSecurityConfig, HandlerType},
    };
    use srv_mod_database::{
        bb8,
        diesel::{Connection, PgConnection},
        diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection},
        diesel_migrations::MigrationHarness,
        migration::MIGRATIONS,
        Pool,
    };
    use srv_mod_handler_base::state::HttpHandlerState;
    use tokio;
    use tower::ServiceExt;

    use super::*;

    fn make_config() -> HandlerConfig {
        let config = HandlerConfig {
            enabled:   true,
            r#type:    HandlerType::Http,
            protocols: vec![handlers::Protocol::Json],
            port:      8081,
            host:      "127.0.0.1".to_string(),
            tls:       None,
            security:  HandlerSecurityConfig {
                encryption_scheme: EncryptionScheme::Plain,
                algorithm:         None,
                encoder:           None,
            },
        };

        config
    }

    async fn drop_database(url: String) {
        let mut connection = PgConnection::establish(url.as_str()).unwrap();

        connection.revert_all_migrations(MIGRATIONS).unwrap();
        connection.run_pending_migrations(MIGRATIONS).unwrap();
    }

    async fn make_pool(url: String) -> Pool {
        let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        Arc::new(
            bb8::Pool::builder()
                .max_size(1u32)
                .build(connection_manager)
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    #[serial]
    async fn test_post_handler_plain_no_encoding() {
        let shared_config = make_config();
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }
        let request = Request::post("/checkin")
            .header("Content-Type", "text/plain")
            .body(Body::from(axum::body::Bytes::from(bytes.to_vec())))
            .unwrap();

        // send the request
        let response = app
            .with_state(route_state.clone())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), false);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_handler_plain_hex() {
        let mut shared_config = make_config();
        shared_config.security.encoder = Some(Encoder::Hex);
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }
        let body = HexEncoder::default().encode(bytes.freeze());

        let request = Request::post("/checkin")
            .header("Content-Type", "text/plain")
            .body(Body::from(body))
            .unwrap();

        // send the request
        let response = app
            .with_state(route_state.clone())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), false);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_handler_plain_base32() {
        let mut shared_config = make_config();
        shared_config.security.encoder = Some(Encoder::Base32);
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }
        let body = Base32Encoder::default().encode(bytes.freeze());

        let request = Request::post("/checkin")
            .header("Content-Type", "text/plain")
            .body(Body::from(body))
            .unwrap();

        // send the request
        let response = app
            .with_state(route_state.clone())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), false);
    }

    #[tokio::test]
    #[serial]
    async fn test_post_handler_plain_base64() {
        let mut shared_config = make_config();
        shared_config.security.encoder = Some(Encoder::Base64);
        let connection_string = "postgresql://kageshirei:kageshirei@localhost/kageshirei".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config:  Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system:  "Windows".to_string(),
            hostname:          "DESKTOP-PC".to_string(),
            domain:            "WORKGROUP".to_string(),
            username:          "user".to_string(),
            ip:                "10.2.123.45".to_string(),
            process_id:        1234,
            parent_process_id: 5678,
            process_name:      "agent.exe".to_string(),
            elevated:          true,
        });
        let checkin = serde_json::to_string(&obj_checkin).unwrap();

        let mut bytes = BytesMut::with_capacity(checkin.len() + magic_numbers::JSON.len());
        for b in magic_numbers::JSON.iter() {
            bytes.put_u8(*b);
        }
        for b in checkin.as_bytes() {
            bytes.put_u8(*b);
        }
        let body = Base64Encoder::default().encode(bytes.freeze());

        let request = Request::post("/checkin")
            .header("Content-Type", "text/plain")
            .body(Body::from(body))
            .unwrap();

        // send the request
        let response = app
            .with_state(route_state.clone())
            .oneshot(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.is_empty(), false);
    }
}

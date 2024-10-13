use axum::{
    body::{Body, Bytes},
    debug_handler,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use rs2_crypt::encoder::{base32::Base32Encoder, base64::Base64Encoder, hex::HexEncoder, Encoder as CryptEncoder};
use srv_mod_config::handlers::{Encoder, EncryptionScheme};
use srv_mod_handler_base::{handle_command_result, state::HandlerSharedState};
use tracing::{instrument, warn};

/// Converts a byte array to a string
fn body_to_string(body: Bytes) -> String { body.iter().map(|b| *b as char).collect::<String>() }

/// Decodes the body of the request based on the encoder or return a failed response
fn decode_or_fail_response(encoder: Encoder, body: Bytes) -> Result<Bytes, Response> {
    match encoder {
        // decode hex, then start processing
        Encoder::Hex => {
            let decoded = HexEncoder::default().decode(body_to_string(body).as_str());

            if decoded.is_err() {
                // if no protocol matches, drop the request
                warn!("Unknown format (not hex), request refused");
                warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

                // always return OK to avoid leaking information
                return Err((StatusCode::OK, "").into_response());
            }

            Ok(decoded.unwrap())
        },
        // decode base32, then start processing
        Encoder::Base32 => {
            let decoded = Base32Encoder::default().decode(body_to_string(body).as_str());

            if decoded.is_err() {
                // if no protocol matches, drop the request
                warn!("Unknown format (not base32), request refused");
                warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

                // always return OK to avoid leaking information
                return Err((StatusCode::OK, "").into_response());
            }

            Ok(decoded.unwrap())
        },
        // decode base64, then start processing
        Encoder::Base64 => {
            let decoded = Base64Encoder::default().decode(body_to_string(body).as_str());

            if decoded.is_err() {
                // if no protocol matches, drop the request
                warn!("Unknown format (not base64), request refused");
                warn!("Internal status code: {}", StatusCode::BAD_REQUEST);

                // always return OK to avoid leaking information
                return Err((StatusCode::OK, "").into_response());
            }

            Ok(decoded.unwrap())
        },
    }
}

/// The handler for the agent checking operation
#[debug_handler]
#[instrument(name = "POST /checkin", skip_all)]
async fn post_handler(State(state): State<HandlerSharedState>, headers: HeaderMap, body: Bytes) -> Response<Body> {
    handle_command_result(state, body, headers, String::new()).await
}

/// Creates the public authentication routes
pub fn route(state: HandlerSharedState) -> Router<HandlerSharedState> {
    Router::new()
        .route("/checkin", post(post_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::Request;
    use bytes::{BufMut, BytesMut};
    use rs2_communication_protocol::{
        communication_structs::checkin::{Checkin, PartialCheckin},
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
            enabled: true,
            r#type: HandlerType::Http,
            protocols: vec![handlers::Protocol::Json],
            port: 8081,
            host: "127.0.0.1".to_string(),
            tls: None,
            security: HandlerSecurityConfig {
                encryption_scheme: EncryptionScheme::Plain,
                algorithm: None,
                encoder: None,
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
        let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config: Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system: "Windows".to_string(),
            hostname: "DESKTOP-PC".to_string(),
            domain: "WORKGROUP".to_string(),
            username: "user".to_string(),
            ip: "10.2.123.45".to_string(),
            process_id: 1234,
            parent_process_id: 5678,
            process_name: "agent.exe".to_string(),
            elevated: true,
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
        let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config: Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system: "Windows".to_string(),
            hostname: "DESKTOP-PC".to_string(),
            domain: "WORKGROUP".to_string(),
            username: "user".to_string(),
            ip: "10.2.123.45".to_string(),
            process_id: 1234,
            parent_process_id: 5678,
            process_name: "agent.exe".to_string(),
            elevated: true,
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
        let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config: Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system: "Windows".to_string(),
            hostname: "DESKTOP-PC".to_string(),
            domain: "WORKGROUP".to_string(),
            username: "user".to_string(),
            ip: "10.2.123.45".to_string(),
            process_id: 1234,
            parent_process_id: 5678,
            process_name: "agent.exe".to_string(),
            elevated: true,
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
        let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

        // Ensure the database is clean
        drop_database(connection_string.clone()).await;
        let pool = make_pool(connection_string.clone()).await;

        let route_state = Arc::new(HttpHandlerState {
            config: Arc::new(shared_config),
            db_pool: pool,
        });
        // init the app router
        let app = route(route_state.clone());

        let obj_checkin = Checkin::new(PartialCheckin {
            operative_system: "Windows".to_string(),
            hostname: "DESKTOP-PC".to_string(),
            domain: "WORKGROUP".to_string(),
            username: "user".to_string(),
            ip: "10.2.123.45".to_string(),
            process_id: 1234,
            parent_process_id: 5678,
            process_name: "agent.exe".to_string(),
            elevated: true,
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

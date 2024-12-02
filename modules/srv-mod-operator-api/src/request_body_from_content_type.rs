//! Extracts the request body based on the content type of the request.

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Form,
    Json,
    RequestExt as _,
};

/// Extracts the request body based on the content type of the request.
/// This does not support `multipart/form-data`, it must be handled separately.
pub struct InferBody<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for InferBody<T>
where
    S: Send + Sync,
    Json<T>: FromRequest<()>,
    Form<T>: FromRequest<()>,
    T: 'static,
{
    type Rejection = Response;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // extract the content type header
        let content_type_header = req.headers().get(CONTENT_TYPE);
        let content_type = content_type_header.and_then(|value| value.to_str().ok());

        // check if the content type is either `application/json` or `application/x-www-form-urlencoded` and
        // extract the payload
        if let Some(content_type) = content_type {
            if content_type.starts_with("application/json") {
                let Json(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }

            if content_type.starts_with("application/x-www-form-urlencoded") {
                let Form(payload) = req.extract().await.map_err(IntoResponse::into_response)?;
                return Ok(Self(payload));
            }
        }

        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, Bytes},
        extract::{Form, Json},
        http::{header::CONTENT_TYPE, Request, StatusCode},
        response::{IntoResponse, Response},
    };
    use hyper::http::HeaderMap;
    use serde::Deserialize;
    use tokio::runtime::Runtime;

    use super::*;

    // Create a simple struct to use for the request body
    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct MyPayload {
        username: String,
        password: String,
    }

    // Mock request creation
    fn mock_json_request(body: &str) -> Request<Body> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        Request::builder()
            .method("POST")
            .uri("/test")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn mock_urlencoded_request(body: &str) -> Request<Body> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        Request::builder()
            .method("POST")
            .uri("/test")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    // Test: handling `application/json` content type
    #[tokio::test]
    #[serial_test::serial]
    async fn test_json_content_type() {
        let json_body = r#"{
            "username": "user1",
            "password": "secret"
        }"#;

        let request = mock_json_request(json_body);

        let body = InferBody::<MyPayload>::from_request(request, &()).await;
        assert!(body.is_ok());

        let InferBody(payload) = body.unwrap();
        assert_eq!(payload.username, "user1");
        assert_eq!(payload.password, "secret");
    }

    // Test: handling `application/x-www-form-urlencoded` content type
    #[tokio::test]
    #[serial_test::serial]
    async fn test_urlencoded_content_type() {
        let urlencoded_body = "username=user1&password=secret";

        let request = mock_urlencoded_request(urlencoded_body);

        let body = InferBody::<MyPayload>::from_request(request, &()).await;
        assert!(body.is_ok());

        let InferBody(payload) = body.unwrap();
        assert_eq!(payload.username, "user1");
        assert_eq!(payload.password, "secret");
    }

    // Test: handling unsupported content type
    #[tokio::test]
    #[serial_test::serial]
    async fn test_unsupported_content_type() {
        let plain_text_body = "This is plain text";

        let mut request = Request::builder()
            .uri("/test")
            .method("POST")
            .header(CONTENT_TYPE, "text/plain")
            .body(Body::from(plain_text_body))
            .unwrap();

        let response = InferBody::<MyPayload>::from_request(request, &()).await;
        assert!(response.is_err());

        let err_response = response.err().unwrap();
        assert_eq!(err_response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}

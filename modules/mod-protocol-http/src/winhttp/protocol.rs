//! The WinHTTP protocol implementation for sending and receiving data.

use alloc::{borrow::ToOwned as _, format, string::String, sync::Arc, vec::Vec};

use kageshirei_communication_protocol::{error::Protocol as ProtocolError, Metadata, Protocol};

use super::client::WinHttpClient;

/// Define the WinHTTP protocol for sending and receiving data.
#[expect(
    clippy::module_name_repetitions,
    reason = "The module name is descriptive and must be kept to this format in order to allow seamless switch \
              between the feature implementations"
)]
pub struct HttpProtocol {
    /// The WinHTTP client used to send requests.
    /// This client maintains session and connection handles for efficient reuse across requests.
    client:   WinHttpClient,
    /// The base URL for the protocol. This is the URL to which requests are sent.
    base_url: String,
}

impl HttpProtocol {
    /// Create a new WinHTTP protocol.
    pub fn new(base_url: String) -> Self {
        Self {
            // Safety: The WinHttpClient::new() function is guaranteed to return a valid WinHttpClient.
            //         given that the winhttp library is correctly loaded in-memory.
            client: unsafe { WinHttpClient::new().unwrap_unchecked() },
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
    fn fill_meta_headers(&mut self, metadata: &Option<Arc<Metadata>>) {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference nor explicitly match against reference"
        )]
        if let Some(meta) = metadata {
            // Add the request ID to the headers.
            // The request ID is a combination of the request ID and the agent ID. This is used by the server to
            // identify the request origin.
            self.client.add_header("X-Request-ID".to_owned(), format!("{}.{}", meta.request_id, meta.agent_id))
                // Add the command ID to the headers.
                // The command ID is used to identify the command that was sent.
                .add_header("X-Identifier".to_owned(), meta.command_id.clone());
        }
    }
}

impl Protocol for HttpProtocol {
    async fn send(&mut self, data: Vec<u8>, metadata: Option<Arc<Metadata>>) -> Result<Vec<u8>, ProtocolError> {
        // Send the request using the WinHTTP client.
        self.fill_meta_headers(&metadata);
        let response = self
            .client
            .post(&self.get_request_url(&metadata), data.to_vec())
            .await?;
        Ok(response)
    }

    async fn receive(&mut self, metadata: Option<Arc<Metadata>>) -> Result<Vec<u8>, ProtocolError> {
        // Send the request using the WinHTTP client.
        self.fill_meta_headers(&metadata);
        let response = self.client.get(&self.get_request_url(&metadata)).await?;
        Ok(response)
    }
}

// #[cfg(test)]
// mod tests {
// use alloc::string::ToString;
//
// use axum::{body::Bytes, http::HeaderMap, routing::get, Router};
// use tokio::select;
// use tokio_util::sync::CancellationToken;
//
// use super::*;
//
// async fn make_dummy_server(cancellation_token: CancellationToken, router: Router<()>, port: u16)
// { let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
// .await
// .unwrap();
//
// axum::serve(listener, router)
// .with_graceful_shutdown(async move {
// select! {
// _ = cancellation_token.cancelled() => {},
// }
// })
// .await
// .unwrap();
// }
//
// async fn handler_post(headers: HeaderMap, body: Bytes) -> String {
// assert_eq!(headers.get("content-type").unwrap(), "text/plain");
// assert_eq!(
// headers.get("X-Request-ID").unwrap(),
// "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
// );
// assert_eq!(
// headers.get("X-Identifier").unwrap(),
// "an3a8hlnrr4638d30yef0oz5sncjdx5w"
// );
// assert_eq!(body.to_vec(), b"bar".to_vec());
//
// "Ok".to_owned()
// }
//
// async fn handler_get(headers: HeaderMap) -> String {
// assert_eq!(
// headers.get("X-Request-ID").unwrap(),
// "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
// );
// assert_eq!(
// headers.get("X-Identifier").unwrap(),
// "an3a8hlnrr4638d30yef0oz5sncjdx5w"
// );
//
// "Ok".to_owned()
// }
//
// #[tokio::test]
// #[serial_test::serial]
// async fn test_send() {
// let cancellation_token = CancellationToken::new();
//
// let router = Router::new().route("/", get(handler_get).post(handler_post));
//
// let call = make_dummy_server(cancellation_token.clone(), router, 8081);
// let server_handle = tokio::spawn(async move {
// call.await;
// });
//
// let mut protocol = HttpProtocol::new("http://localhost:8001".to_string());
//
// let data = b"bar".to_vec();
// let result = protocol
// .send(
// data,
// Some(Arc::new(Metadata {
// request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
// agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
// command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
// path:       None,
// })),
// )
// .await;
//
// if let Err(e) = result {
// match e {
// ProtocolError::SendingError(s) => {
// panic!("SendingError: {}", unsafe { s.unwrap_unchecked() });
// },
// ProtocolError::ReceivingError(s) => {
// panic!("ReceivingError: {}", unsafe { s.unwrap_unchecked() });
// },
// ProtocolError::InitializationError(s) => {
// panic!("InitializationError: {}", s);
// },
// ProtocolError::Generic(s) => {
// panic!("Generic: {}", s);
// },
// ProtocolError::ConnectionError => {
// panic!("ConnectionError: ");
// },
// ProtocolError::DisconnectionError => {
// panic!("DisconnectionError: ");
// },
// ProtocolError::MessageError => {
// panic!("MessageError: ");
// },
// ProtocolError::ReceiveMessageError => {
// panic!("ReceiveMessageError: ");
// },
// }
// }
// if let Ok(result) = result {
// panic!("Result: {:?}", result);
// }
//
// assert_eq!(result, b"Ok".to_vec());
//
// cancellation_token.cancel();
// server_handle.await.unwrap();
// }
//
// #[tokio::test]
// #[serial_test::serial]
// async fn test_receive() {
// let cancellation_token = CancellationToken::new();
//
// let router = Router::new().route("/", get(handler_get).post(handler_post));
//
// let call = make_dummy_server(cancellation_token.clone(), router, 8080);
// let server_handle = tokio::spawn(async move {
// call.await;
// });
//
// let mut protocol = HttpProtocol::new("http://localhost:8080".to_string());
//
// let result = protocol
// .receive(Some(Arc::new(Metadata {
// request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
// agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
// command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
// path:       None,
// })))
// .await;
//
// let result = unsafe { result.unwrap_unchecked() };
// assert_eq!(result, b"Ok".to_vec());
//
// cancellation_token.cancel();
// server_handle.await.unwrap();
// }
// }

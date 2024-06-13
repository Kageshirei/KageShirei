use std::sync::Arc;

use serde::Serialize;
use tokio::sync::broadcast::Sender;
use tracing::error;

use srv_mod_database::models::log::Log;
use srv_mod_database::models::notification::Notification;

#[derive(Debug, Clone, Serialize)]
pub enum Events {
	/// A log event
	Log(Log),
	/// A notification event
	Notification(Notification),
	/// A command output event, this will be emitted only by size-restricted protocols (e.g. DNS, NTP, etc.) which allows
	/// for a limited amount of data to be sent to the server in a single request.
	CommandOutput(CommandOutputEvent),
}

/// The event for command output, this is basically the command struct with fewer fields
#[derive(Debug, Clone, Serialize)]
pub struct CommandOutputEvent {
	/// The unique identifier for the command (cuid2)
	pub id: String,
	/// The command's output
	pub output: String,
	/// The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
	/// was ran in the context of the global terminal emulator.
	pub session_id: String,
	/// The command's exit code
	pub exit_code: Option<i32>,
}

impl Events {
	/// Emit the event
	///
	/// # Arguments
	///
	/// * `sender` - The sender to send the event to
	/// * `event` - The event to send
	///
	/// # Example
	///
	/// ```
	/// use std::sync::Arc;
	/// use tokio::sync::broadcast::Sender;
	/// use srv_mod_operator_api::events::{CommandOutputEvent, Events};
	///
	/// let (sender, _) = tokio::sync::broadcast::channel(128);
	/// Events::emit(sender, Events::CommandOutput(CommandOutputEvent {
	/// 	id: "123".to_string(),
	/// 	output: "Hello, World!".to_string(),
	/// 	session_id: "456".to_string(),
	/// 	exit_code: Some(0)
	/// }));
	/// ```
	pub fn emit(sender: Sender<Arc<Events>>, event: Events) {
		if let Err(e) = sender.send(Arc::new(event)) {
			error!("Error sending event: {:?}", e);
		}
	}
}
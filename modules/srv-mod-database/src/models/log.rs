use diesel::{AsChangeset, Insertable, Queryable, Selectable};

use crate::CUID2;
use crate::schema_extension::LogLevel;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::schema::logs)]
pub struct Log {
	/// The unique identifier for the log (cuid2)
	pub id: String,
	/// The log level
	pub level: LogLevel,
	/// The log message
	pub message: Option<String>,
	/// The log title
	pub title: Option<String>,
	/// The log extra data (JSON) for additional information
	pub extra: Option<serde_json::Value>,
	/// The log's creation date
	pub created_at: chrono::DateTime<chrono::Utc>,
	/// The log's last update date
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::logs)]
pub struct CreateLog {
	/// The unique identifier for the log (cuid2)
	pub id: String,
	/// The log level
	pub level: LogLevel,
	/// The log message
	pub message: Option<String>,
	/// The log title
	pub title: Option<String>,
	/// The log extra data (JSON) for additional information
	pub extra: Option<serde_json::Value>,
}

impl CreateLog {
	pub fn new(level: LogLevel) -> Self {
		Self {
			id: CUID2.create_id(),
			level,
			message: None,
			title: None,
			extra: None,
		}
	}
}
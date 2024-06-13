use diesel::{AsChangeset, Insertable, Queryable, Selectable, SelectableHelper};
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::CUID2;
use crate::schema_extension::LogLevel;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq, Serialize, Deserialize)]
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
	/// Create a new log
	pub fn new(level: LogLevel) -> Self {
		Self {
			id: CUID2.create_id(),
			level,
			message: None,
			title: None,
			extra: None,
		}
	}

	/// Set the log message
	pub fn with_message(mut self, message: impl Into<String>) -> Self {
		self.message = Some(message.into());
		self
	}

	/// Set the log title
	pub fn with_title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Set the log extra data
	pub fn with_extra<T>(mut self, extra: T) -> Self
	where
		T: serde::Serialize,
	{
		self.extra = Some(serde_json::to_value(extra).unwrap());
		self
	}

	pub fn with_extra_value(mut self, extra: serde_json::Value) -> Self {
		self.extra = Some(extra);
		self
	}

	/// Save the log to the database
	pub async fn save(&self, mut conn: &mut AsyncPgConnection) -> Result<Log, Error> {
		diesel::insert_into(crate::schema::logs::table)
			.values(self)
			.returning(Log::as_select())
			.get_result(&mut conn)
			.await
	}
}
use diesel::{AsChangeset, Insertable, Queryable, Selectable};

use crate::CUID2;
use crate::schema_extension::LogLevel;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::schema::notifications)]
pub struct Notification {
	/// The unique identifier for the notification (cuid2)
	pub id: String,
	/// The notification level
	pub level: LogLevel,
	/// The notification message
	pub message: String,
	/// The notification title
	pub title: String,
	/// The notification's creation date
	pub created_at: chrono::DateTime<chrono::Utc>,
	/// The notification's last update date
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::notifications)]
pub struct CreateNotification {
	/// The unique identifier for the notification (cuid2)
	pub id: String,
	/// The notification level
	pub level: LogLevel,
	/// The notification message
	pub message: String,
	/// The notification title
	pub title: String,
}

impl CreateNotification {
	pub fn new(level: LogLevel) -> Self {
		Self {
			id: CUID2.create_id(),
			level,
			message: "".to_string(),
			title: "".to_string(),
		}
	}
}
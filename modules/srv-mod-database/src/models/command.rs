use diesel::{AsChangeset, Insertable, Queryable, Selectable};

use crate::CUID2;

#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::schema::commands)]
pub struct Command {
	/// The unique identifier for the command (cuid2)
	pub id: String,
	/// Who ran the command
	pub ran_by: String,
	/// The command that was run
	pub command: String,
	/// The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
	/// was ran in the context of the global terminal emulator.
	pub session_id: String,
	/// The command's output
	pub output: Option<String>,
	/// The command's exit code
	pub exit_code: Option<i32>,
	/// Soft delete for the `clear` command.
	pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
	/// Restore timestamp for the `history` command.
	pub restored_at: Option<chrono::DateTime<chrono::Utc>>,
	/// The command's creation date
	pub created_at: chrono::DateTime<chrono::Utc>,
	/// The command's last update date
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::commands)]
pub struct CreateCommand {
	/// The unique identifier for the command (cuid2)
	pub id: String,
	/// Who ran the command
	pub ran_by: String,
	/// The command that was run
	pub command: String,
	/// The session_id is a foreign key that references the agents table or a static value defined to "global" if the command
	/// was ran in the context of the global terminal emulator.
	pub session_id: String,
	/// The command's output
	pub output: Option<String>,
	/// The command's exit code
	pub exit_code: Option<i32>,
	/// Soft delete for the `clear` command.
	pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
	/// Restore timestamp for the `history` command.
	pub restored_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl CreateCommand {
	pub fn new(ran_by: String, session_id: String) -> Self {
		Self {
			id: CUID2.create_id(),
			ran_by,
			command: "".to_string(),
			session_id,
			output: None,
			exit_code: None,
			deleted_at: None,
			restored_at: None,
		}
	}
}
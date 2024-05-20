use chrono::Utc;
use diesel::prelude::*;

/// The User model, identifies a RS2 operator
#[derive(Debug, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = crate::database::schema::users)]
pub struct User {
	/// The unique identifier for the user
	pub id: uuid::Uuid,
	/// The username for the user (unique)
	pub username: String,
	/// The user's password (hashed)
	pub password: String,
	// Creation date of the user
	pub created_at: chrono::DateTime<Utc>,
	// Last time the user was updated
	pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::database::schema::users)]
pub struct CreateUser {
	pub username: String,
	pub password: String,
}

impl CreateUser {
	pub fn new(username: String, password: String) -> Self {
		Self {
			username,
			password: rs2_crypt::argon::Argon2::hash_password(password.as_str()).unwrap(),
		}
	}
}
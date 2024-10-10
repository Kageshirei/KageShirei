pub mod tests {
    use std::sync::Arc;

    use srv_mod_database::{bb8, diesel, Pool};
    use srv_mod_database::diesel::{Connection, PgConnection, SelectableHelper};
    use srv_mod_database::diesel_async::{AsyncPgConnection, RunQueryDsl};
    use srv_mod_database::diesel_async::pooled_connection::AsyncDieselConnectionManager;
    use srv_mod_database::diesel_migrations::MigrationHarness;
    use srv_mod_database::migration::MIGRATIONS;
    use srv_mod_database::models::user::{CreateUser, User};
    use srv_mod_database::schema::users;

    pub async fn drop_database() {
		let mut connection = PgConnection::establish("postgresql://rs2:rs2@localhost/rs2").unwrap();

		connection.revert_all_migrations(MIGRATIONS).unwrap();
		connection.run_pending_migrations(MIGRATIONS).unwrap();
	}

	pub async fn make_pool() -> Pool {
		let connection_manager =
			AsyncDieselConnectionManager::<AsyncPgConnection>::new("postgresql://rs2:rs2@localhost/rs2");

		Arc::new(
			bb8::Pool::builder()
				.max_size(1u32)
				.build(connection_manager)
				.await
				.unwrap(),
		)
	}

	pub async fn generate_test_user(pool: Pool) -> User {
		let mut connection = pool.get().await.unwrap();
		diesel::insert_into(users::table)
			.values(CreateUser::new("test".to_string(), "test".to_string()))
			.returning(User::as_select())
			.get_result(&mut connection)
			.await
			.unwrap()
	}
}

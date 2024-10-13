pub mod tests {
    use std::sync::Arc;

    use srv_mod_database::{
        bb8,
        diesel,
        diesel::{Connection, PgConnection, SelectableHelper},
        diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl},
        diesel_migrations::MigrationHarness,
        migration::MIGRATIONS,
        models::user::{CreateUser, User},
        schema::users,
        Pool,
    };

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

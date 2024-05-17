use diesel::RunQueryDsl;
use log::info;

use crate::cli::generate::operator::GenerateOperatorArguments;
use crate::config::config::SharedConfig;
use crate::database::models::user::CreateUser;

pub async fn generate_operator(args: &GenerateOperatorArguments, config: SharedConfig) -> anyhow::Result<()> {
	use crate::database::schema::users::dsl::*;

	let readonly_config = config.read().await;

	// run the migrations on server startup as we need to ensure the database is up-to-date before we can insert the
	// operator.
	// It is safe here to unpack and the option in an unique statement as if the migration fails, the program will exit
	let Some(mut connection) = crate::database::migration::run_pending(readonly_config.database.url.as_str(), true)?
		else { unreachable!() };

	let user_id = diesel::insert_into(users)
		.values(CreateUser::new(args.username.clone(), args.password.clone()))
		.returning(id)
		.get_result::<uuid::Uuid>(&mut connection)?;

	info!("Created operator with id: {}", user_id);
	Ok(())
}
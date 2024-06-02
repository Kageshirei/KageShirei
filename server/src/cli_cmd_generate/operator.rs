use log::{error, info};

use srv_mod_config::SharedConfig;
use srv_mod_database::diesel;
use srv_mod_database::models::user::CreateUser;

use crate::cli::generate::operator::GenerateOperatorArguments;

pub async fn generate_operator(
	args: &GenerateOperatorArguments,
	config: SharedConfig,
) -> anyhow::Result<()> {
	use srv_mod_database::schema::users::dsl::*;
	use srv_mod_database::diesel::prelude::*;

	let readonly_config = config.read().await;

	// run the migrations on server startup as we need to ensure the database is up-to-date before we can insert the
	// operator.
	// It is safe here to unpack and the option in an unique statement as if the migration fails, the program will exit
	let Some(mut connection) =
		srv_mod_database::migration::run_pending(readonly_config.database.url.as_str(), true)?
		else {
			unreachable!()
		};

	let user_exists = diesel::select(diesel::dsl::exists(
		users.filter(username.eq(&args.username)),
	))
		.get_result::<bool>(&mut connection)?;

	if user_exists {
		error!("Operator with username '{}' already exists", args.username);
		return Err(anyhow::anyhow!("User already exists"));
	}

	// Insert the operator into the database
	let user_id = diesel::insert_into(users)
		.values(CreateUser::new(
			args.username.clone(),
			args.password.clone(),
		))
		.returning(id)
		.get_result::<String>(&mut connection);

	// If the user_id is an error, log the error and exit
	if let Err(e) = user_id {
		error!("Something went wrong, {}", e);

		return Err(anyhow::anyhow!("Failed to create operator"));
	}

	let user_id = user_id.unwrap();

	info!("Created operator with id: {}", user_id);
	Ok(())
}

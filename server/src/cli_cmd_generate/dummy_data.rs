use log::{error, info};

use srv_mod_config::SharedConfig;
use srv_mod_database::{CUID2, diesel};
use srv_mod_database::diesel::prelude::*;
use srv_mod_database::models::agent::{Agent, CreateAgent};
use srv_mod_database::models::agent_profile::{AgentProfile, CreateAgentProfile};
use srv_mod_database::models::filter::CreateFilter;
use srv_mod_database::models::user::CreateUser;
use srv_mod_database::schema::agent_profiles;
use srv_mod_database::schema::agents;
use srv_mod_database::schema::filters;
use srv_mod_database::schema::users;
use srv_mod_database::schema_extension::{AgentFields, FilterOperator};

use crate::cli::generate::operator::GenerateOperatorArguments;

pub async fn make_dummy_data(
	config: SharedConfig,
) -> anyhow::Result<()> {
	let readonly_config = config.read().await;

	// run the migrations on server startup as we need to ensure the database is up-to-date before we can insert the
	// operator.
	// It is safe here to unpack and the option in an unique statement as if the migration fails, the program will exit
	let Some(mut connection) = srv_mod_database::migration::reset_all(
		readonly_config.database.url.as_str(), true,
	)?
	else {
		unreachable!()
	};

	// Insert the operator into the database
	let user_id = diesel::insert_into(users::table)
		.values(CreateUser::new(
			"test".to_string(),
			"test".to_string(),
		))
		.returning(users::id)
		.get_result::<String>(&mut connection);

	info!("Test user inserted, authenticate using test:test");

	// insert agents
	let agent = diesel::insert_into(agents::table)
		.values(CreateAgent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			integrity_level: 8192,
			cwd: "C:\\Users\\user".to_string(),
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature-0".to_string(),
		})
		.returning(Agent::as_returning())
		.get_result::<Agent>(&mut connection)
		.unwrap();

	let profile = diesel::insert_into(agent_profiles::table)
		.values(CreateAgentProfile {
			id: CUID2.create_id(),
			name: "Test Profile".to_string(),
			kill_date: Some(1),
			working_hours: Some(vec![1, 1]),
			polling_interval: Some(1),
			polling_jitter: Some(1),
		})
		.returning(AgentProfile::as_returning())
		.get_result::<AgentProfile>(&mut connection)
		.unwrap();

	diesel::insert_into(filters::table)
		.values(&vec![
			CreateFilter {
				id: CUID2.create_id(),
				agent_profile_id: profile.id.clone(),
				agent_field: AgentFields::Hostname,
				filter_op: FilterOperator::Equals,
				value: "DESKTOP-PC".to_string(),
				sequence: 1,
				next_hop_relation: None,
				grouping_start: false,
				grouping_end: false,
			},
		])
		.execute(&mut connection)
		.unwrap();

	Ok(())
}

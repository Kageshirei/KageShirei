use rs2_communication_protocol::communication_structs::checkin::CheckinResponse;
use srv_mod_database::diesel::ExpressionMethods;
use srv_mod_database::diesel::QueryDsl;
use srv_mod_database::diesel_async::RunQueryDsl;
use srv_mod_database::models::agent::Agent;
use srv_mod_database::models::agent_profile::AgentProfile;
use srv_mod_database::models::filter::Filter;
use srv_mod_database::Pool;
use srv_mod_database::schema::agent_profiles::dsl::agent_profiles;
use srv_mod_database::schema_extension::{FilterOperator, LogicalOperator};

struct GroupEvaluationResult {
	result: bool,
	next_hop: Option<LogicalOperator>,
	hops: usize,
}

/// Evaluate a filter and return the result
fn evaluate_filter(agent: &Agent, filter: &Filter) -> bool {
	match filter.filter_op {
		FilterOperator::Equals => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			v.as_str() == filter.value.as_str()
		}
		FilterOperator::NotEquals => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			v.as_str() != filter.value.as_str()
		}
		FilterOperator::Contains => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			v.contains(filter.value.as_str())
		}
		FilterOperator::NotContains => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			!v.contains(filter.value.as_str())
		}
		FilterOperator::StartsWith => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			v.starts_with(filter.value.as_str())
		}
		FilterOperator::EndsWith => {
			let v = agent.get_field_value(&filter.agent_field).unwrap_or(String::new());
			v.ends_with(filter.value.as_str())
		}
	}
}

/// Combine the results of a filter with the next_hop_relation
fn combine_results(result: Option<bool>, next_hop: Option<&LogicalOperator>, intermediary_result: bool) -> bool {
	if result.is_none() {
		return intermediary_result;
	}

	if next_hop.is_none() {
		return intermediary_result;
	}

	match next_hop.unwrap() {
		LogicalOperator::And => {
			result.unwrap() && intermediary_result
		}
		LogicalOperator::Or => {
			result.unwrap() || intermediary_result
		}
	}
}

/// Evaluate a group of filters and exits returning the result once a group end is found or the filters are exhausted
fn evaluate_group(agent: &Agent, filters: Vec<Filter>, index: usize) -> GroupEvaluationResult {
	// init the result container
	let mut result: Option<bool> = None;
	let mut next_hop: Option<LogicalOperator> = None;
	let original_index = index;
	let mut i = index;

	while i < filters.len() {
		let filter = &filters[i];

		let intermediary_result: bool;
		let mut pending_next_hop: Option<LogicalOperator> = None;

		if filter.grouping_start {
			let group_start_result = evaluate_filter(agent, filter);
			let evaluation_result = evaluate_group(agent, filters.clone(), i + 1);
			intermediary_result = combine_results(Some(group_start_result), filter.next_hop_relation.as_ref(), evaluation_result.result);
			pending_next_hop = evaluation_result.next_hop;

			// skip the hops already evaluated
			i += evaluation_result.hops;
		} else {
			// apply the filter
			intermediary_result = evaluate_filter(agent, filter);
		}

		// if the result is None, set the result to the intermediary result
		if result.is_none() {
			result = Some(intermediary_result);
		} else {
			result = Some(combine_results(result, next_hop.as_ref(), intermediary_result));
		}

		if pending_next_hop.is_some() {
			next_hop = pending_next_hop.clone();
		} else {
			// if the filter has a next_hop_relation, set the next_hop_relation to the filter's next_hop_relation
			if filter.next_hop_relation.is_some() {
				next_hop = filter.next_hop_relation.clone();
				pending_next_hop = filter.next_hop_relation.clone();
			}
		}

		// if the group ends, return the result, group ending has precedence over the next_hop_relation, this means that
		// (random_queries... and) equals (random_queries...)
		if filter.grouping_end {
			return GroupEvaluationResult {
				result: result.unwrap_or(false),
				next_hop: pending_next_hop,
				hops: i + 1 - original_index,
			};
		}

		i += 1;
	}

	// if the group ends, return the result
	return GroupEvaluationResult {
		result: result.unwrap_or(false),
		next_hop: None,
		hops: filters.len() - original_index,
	};
}

/// Apply filters to the agent and return the configuration profile
pub async fn apply_filters(agent: &Agent, db_pool: Pool) -> CheckinResponse {
	use srv_mod_database::schema::filters::dsl::*;
	use srv_mod_database::schema::agent_profiles::dsl as agent_dsl;

	let mut connection = db_pool.get().await.unwrap();

	// get all the agent profiles, ordered by creation date, descending, so the latest profile is first
	let available_profiles = agent_profiles.order_by(agent_dsl::created_at.desc())
	                                       .get_results::<AgentProfile>(&mut connection)
	                                       .await;

	// if there are no profiles or an error occurred, return the default values
	if available_profiles.is_err() || available_profiles.as_ref().unwrap().is_empty() {
		return CheckinResponse {
			id: agent.id.clone(),
			working_hours: None,
			kill_date: None,
			polling_jitter: 10_000, // 10 seconds of jitter (polling range from 20 to 40 seconds)
			polling_interval: 30_000, // 30 seconds of polling interval
		};
	}
	let available_profiles = available_profiles.unwrap();

	for profile in available_profiles.iter() {
		let profile_filters = filters.filter(agent_profile_id.eq(profile.id.clone()))
		                             .order_by(sequence)
		                             .get_results::<Filter>(&mut connection)
		                             .await
		                             .unwrap();

		// if the result is Some, return the profile configuration
		let final_result = evaluate_group(agent, profile_filters, 0);
		if final_result.result {
			return CheckinResponse {
				id: agent.id.clone(),
				working_hours: profile.working_hours.clone(),
				kill_date: profile.kill_date.clone(),
				polling_jitter: profile.polling_jitter
				                       .unwrap_or(10_000),
				polling_interval: profile.polling_interval
				                         .unwrap_or(30_000),
			};
		}
	}

	// fallback return type if none of the filters match
	return CheckinResponse {
		id: agent.id.clone(),
		working_hours: None,
		kill_date: None,
		polling_jitter: 10_000, // 10 seconds of jitter (polling range from 20 to 40 seconds)
		polling_interval: 30_000, // 30 seconds of polling interval
	};
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use serial_test::serial;

	use srv_mod_config::handlers;
	use srv_mod_config::handlers::{EncryptionScheme, HandlerConfig, HandlerSecurityConfig, HandlerType};
	use srv_mod_database::{bb8, CUID2, diesel, Pool};
	use srv_mod_database::diesel::{Connection, PgConnection, SelectableHelper};
	use srv_mod_database::diesel_async::AsyncPgConnection;
	use srv_mod_database::diesel_async::pooled_connection::AsyncDieselConnectionManager;
	use srv_mod_database::diesel_migrations::MigrationHarness;
	use srv_mod_database::migration::MIGRATIONS;
	use srv_mod_database::models::agent_profile::CreateAgentProfile;
	use srv_mod_database::models::filter::CreateFilter;
	use srv_mod_database::schema::filters::dsl::filters;
	use srv_mod_database::schema_extension::AgentFields;
	use srv_mod_handler_base::state::HttpHandlerState;

	use super::*;

	fn make_config() -> HandlerConfig {
		let config = HandlerConfig {
			enabled: true,
			r#type: HandlerType::Http,
			protocols: vec![
				handlers::Protocol::Json
			],
			port: 8081,
			host: "127.0.0.1".to_string(),
			tls: None,
			security: HandlerSecurityConfig {
				encryption_scheme: EncryptionScheme::Plain,
				algorithm: None,
				encoder: None,
			},
		};

		config
	}

	async fn drop_database(url: String) {
		let mut connection = PgConnection::establish(url.as_str()).unwrap();

		connection.revert_all_migrations(MIGRATIONS).unwrap();
		connection.run_pending_migrations(MIGRATIONS).unwrap();
	}

	async fn make_pool(url: String) -> Pool {
		let connection_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
		Arc::new(
			bb8::Pool::builder()
				.max_size(1u32)
				.build(connection_manager)
				.await
				.unwrap(),
		)
	}

	#[tokio::test]
	#[serial]
	async fn test_apply_filters_simple_true_check() {
		let agent = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};

		let shared_config = make_config();
		let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
		let pool = make_pool(connection_string.clone()).await;

		let route_state = Arc::new(HttpHandlerState {
			config: Arc::new(shared_config),
			db_pool: pool,
		});

		let profile = diesel::insert_into(agent_profiles)
			.values(CreateAgentProfile {
				id: CUID2.create_id(),
				name: "Test Profile".to_string(),
				kill_date: Some(1),
				working_hours: Some(vec![1, 1]),
				polling_interval: Some(1),
				polling_jitter: Some(1),
			})
			.returning(AgentProfile::as_returning())
			.get_result::<AgentProfile>(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		diesel::insert_into(filters)
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
			.execute(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		let response = apply_filters(&agent, &route_state).await;

		println!("{:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		drop_database(connection_string.clone()).await;
	}

	#[tokio::test]
	#[serial]
	async fn test_apply_filters_simple_false_check() {
		let agent = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "example-hostname".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};

		let shared_config = make_config();
		let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
		let pool = make_pool(connection_string.clone()).await;

		let route_state = Arc::new(HttpHandlerState {
			config: Arc::new(shared_config),
			db_pool: pool,
		});

		let profile = diesel::insert_into(agent_profiles)
			.values(CreateAgentProfile {
				id: CUID2.create_id(),
				name: "Test Profile".to_string(),
				kill_date: Some(1),
				working_hours: Some(vec![1, 1]),
				polling_interval: Some(1),
				polling_jitter: Some(1),
			})
			.returning(AgentProfile::as_returning())
			.get_result::<AgentProfile>(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		diesel::insert_into(filters)
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
			.execute(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		let response = apply_filters(&agent, &route_state).await;

		println!("{:#?}", response);
		assert_eq!(response.kill_date, None);
		assert_eq!(response.working_hours, None);
		assert_eq!(response.polling_interval, 30_000);
		assert_eq!(response.polling_jitter, 10_000);

		drop_database(connection_string.clone()).await;
	}

	#[tokio::test]
	#[serial]
	async fn test_apply_filters_one_level_complex_check() {
		let shared_config = make_config();
		let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
		let pool = make_pool(connection_string.clone()).await;

		let route_state = Arc::new(HttpHandlerState {
			config: Arc::new(shared_config),
			db_pool: pool,
		});

		let profile = diesel::insert_into(agent_profiles)
			.values(CreateAgentProfile {
				id: CUID2.create_id(),
				name: "Test Profile".to_string(),
				kill_date: Some(1),
				working_hours: Some(vec![1, 1]),
				polling_interval: Some(1),
				polling_jitter: Some(1),
			})
			.returning(AgentProfile::as_returning())
			.get_result::<AgentProfile>(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		// NOTE: the following syntax may be prone to misunderstanding as parenthesis are not used to group the filters,
		// using parenthesis is recommended to avoid confusion.
		// hostname = "DESKTOP-PC" AND operative_system = "Windows" or ip = "1.1.1.1"
		// THIS IS EQUAL to (hostname = "DESKTOP-PC" AND operative_system = "Windows") or ip = "1.1.1.1"
		// THIS IS NOT EQUAL to hostname = "DESKTOP-PC" AND (operative_system = "Windows" or ip = "1.1.1.1")
		// Use parenthesis if the latter is the intended behavior
		let or_id = CUID2.create_id();
		let and_id = CUID2.create_id();
		diesel::insert_into(filters)
			.values(&vec![
				// hostname = "DESKTOP-PC"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::Hostname,
					filter_op: FilterOperator::Equals,
					value: "DESKTOP-PC".to_string(),
					sequence: 1,
					next_hop_relation: Some(LogicalOperator::And),
					grouping_start: false,
					grouping_end: false,
				},
				// operative_system = "Windows"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::OperativeSystem,
					filter_op: FilterOperator::Equals,
					value: "Windows".to_string(),
					sequence: 2,
					next_hop_relation: Some(LogicalOperator::Or),
					grouping_start: false,
					grouping_end: false,
				},
				// ip = "1.1.1.1"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::Ip,
					filter_op: FilterOperator::Equals,
					value: "1.1.1.1".to_string(),
					sequence: 3,
					next_hop_relation: None,
					grouping_start: false,
					grouping_end: false,
				},
			])
			.execute(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		let agent_matching_hostname_and_os = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname_and_os, &route_state).await;

		println!("agent_matching_hostname_and_os (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		let agent_matching_hostname = Agent {
			id: CUID2.create_id(),
			operative_system: "Linux".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "0.0.0.0".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname, &route_state).await;

		println!("agent_matching_hostname (expected fallback): {:#?}", response);
		assert_eq!(response.kill_date, None);
		assert_eq!(response.working_hours, None);
		assert_eq!(response.polling_interval, 30_000);
		assert_eq!(response.polling_jitter, 10_000);

		let agent_matching_hostname_and_ip = Agent {
			id: CUID2.create_id(),
			operative_system: "Linux".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname_and_ip, &route_state).await;

		println!("agent_matching_hostname_and_ip (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		let agent_matching_os_and_ip = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "example-hostname".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_os_and_ip, &route_state).await;

		println!("agent_matching_hostname_and_ip (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		drop_database(connection_string.clone()).await;
	}

	#[tokio::test]
	#[serial]
	async fn test_apply_filters_nesting_complex_check() {
		let shared_config = make_config();
		let connection_string = "postgresql://rs2:rs2@localhost/rs2".to_string();

		// Ensure the database is clean
		drop_database(connection_string.clone()).await;
		let pool = make_pool(connection_string.clone()).await;

		let route_state = Arc::new(HttpHandlerState {
			config: Arc::new(shared_config),
			db_pool: pool,
		});

		let profile = diesel::insert_into(agent_profiles)
			.values(CreateAgentProfile {
				id: CUID2.create_id(),
				name: "Test Profile".to_string(),
				kill_date: Some(1),
				working_hours: Some(vec![1, 1]),
				polling_interval: Some(1),
				polling_jitter: Some(1),
			})
			.returning(AgentProfile::as_returning())
			.get_result::<AgentProfile>(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		// operative_system equals "Windows" or hostname equals "DESKTOP-PC";
		//
		// (process_name starts_with "example" and process_name ends_with ".exe") or ip not_equals "1.1.1.1";

		// (hostname = "DESKTOP-PC" AND operative_system = "Windows") or ip = "1.1.1.1"
		let or_id = CUID2.create_id();
		let and_id = CUID2.create_id();
		diesel::insert_into(filters)
			.values(&vec![
				// hostname = "DESKTOP-PC"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::Hostname,
					filter_op: FilterOperator::Equals,
					value: "DESKTOP-PC".to_string(),
					sequence: 1,
					next_hop_relation: Some(LogicalOperator::And),
					grouping_start: true,
					grouping_end: false,
				},
				// operative_system = "Windows"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::OperativeSystem,
					filter_op: FilterOperator::Equals,
					value: "Windows".to_string(),
					sequence: 2,
					next_hop_relation: Some(LogicalOperator::Or),
					grouping_start: false,
					grouping_end: true,
				},
				// ip = "1.1.1.1"
				CreateFilter {
					id: CUID2.create_id(),
					agent_profile_id: profile.id.clone(),
					agent_field: AgentFields::Ip,
					filter_op: FilterOperator::Equals,
					value: "1.1.1.1".to_string(),
					sequence: 3,
					next_hop_relation: None,
					grouping_start: false,
					grouping_end: false,
				},
			])
			.execute(&mut route_state.db_pool.get().await.unwrap())
			.await
			.unwrap();

		let agent_matching_hostname_and_os = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname_and_os, &route_state).await;

		println!("agent_matching_hostname_and_os (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		let agent_matching_hostname = Agent {
			id: CUID2.create_id(),
			operative_system: "Linux".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "0.0.0.0".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname, &route_state).await;

		println!("agent_matching_hostname (expected fallback): {:#?}", response);
		assert_eq!(response.kill_date, None);
		assert_eq!(response.working_hours, None);
		assert_eq!(response.polling_interval, 30_000);
		assert_eq!(response.polling_jitter, 10_000);

		let agent_matching_hostname_and_ip = Agent {
			id: CUID2.create_id(),
			operative_system: "Linux".to_string(),
			hostname: "DESKTOP-PC".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_hostname_and_ip, &route_state).await;

		println!("agent_matching_hostname_and_ip (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		let agent_matching_os_and_ip = Agent {
			id: CUID2.create_id(),
			operative_system: "Windows".to_string(),
			hostname: "example-hostname".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_os_and_ip, &route_state).await;

		println!("agent_matching_os_and_ip (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		let agent_matching_ip = Agent {
			id: CUID2.create_id(),
			operative_system: "Linux".to_string(),
			hostname: "example-hostname".to_string(),
			domain: "example-domain".to_string(),
			username: "user".to_string(),
			ip: "1.1.1.1".to_string(),
			process_id: 1234,
			parent_process_id: 5678,
			process_name: "example.exe".to_string(),
			elevated: false,
			server_secret_key: "server-key".to_string(),
			secret_key: "key".to_string(),
			signature: "signature".to_string(),
			created_at: Default::default(),
			updated_at: Default::default(),
		};
		let response = apply_filters(&agent_matching_ip, &route_state).await;

		println!("agent_matching_ip (expected profile): {:#?}", response);
		assert_eq!(response.kill_date, Some(1));
		assert_eq!(response.working_hours, Some(vec![Some(1), Some(1)]));
		assert_eq!(response.polling_interval, 1);
		assert_eq!(response.polling_jitter, 1);

		drop_database(connection_string.clone()).await;
	}
}
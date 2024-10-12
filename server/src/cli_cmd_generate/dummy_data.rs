use crate::auto_migrate;
use crate::cli::generate::operator::GenerateOperatorArguments;
use log::{error, info};
use rs2_communication_protocol::network_interface::NetworkInterface;
use srv_mod_config::SharedConfig;
use srv_mod_entity::active_enums::{AgentField, AgentIntegrity, FilterOperation};
use srv_mod_entity::entities::{agent, agent_profile, filter, user};
use srv_mod_entity::sea_orm::prelude::*;
use srv_mod_entity::sea_orm::sqlx::types::chrono::{NaiveDate, NaiveTime, Utc};
use srv_mod_entity::sea_orm::ActiveValue::Set;

pub async fn make_dummy_data(config: SharedConfig) -> Result<(), String> {
    let readonly_config = config.read().await;
    let db = auto_migrate::run(&readonly_config.database.url, &readonly_config).await?;

    // Insert the operator into the database
    let new_user = user::ActiveModel {
        username: Set("test".to_string()),
        password: Set(rs2_crypt::argon::Argon2::hash_password("test")?),
        ..Default::default()
    };

    new_user.insert(&db).await.map_err(|e| {
        error!("Failed to create operator: {}", e);
        "Failed to create operator".to_string()
    })?;

    info!("Test user inserted, authenticate using test:test");

    // insert agents
    let new_agent = agent::ActiveModel {
        operating_system: Set("Windows".to_string()),
        hostname: Set("DESKTOP-PC".to_string()),
        domain: Set(Some("example-domain".to_string())),
        username: Set("user".to_string()),
        network_interfaces: Set(vec![
            NetworkInterface {
                name: Some("Ethernet".to_string()),
                address: Some("1.2.3.4".to_string()),
                dhcp_server: Some("10.1.2.3".to_string()),
            }
        ]),
        pid: Set(1234),
        ppid: Set(5678),
        process_name: Set("example.exe".to_string()),
        integrity: Set(AgentIntegrity::Medium),
        cwd: Set("C:\\Users\\user".to_string()),
        server_secret: Set("server-key".to_string()),
        secret: Set("key".to_string()),
        signature: Set("signature-0".to_string()),
        ..Default::default()
    };
    let agent = new_agent.insert(&db).await.map_err(|e| {
        error!("Failed to create agent: {}", e);
        "Failed to create agent".to_string()
    })?;

    info!("Test agent inserted, id: {}", agent.id);

    let new_profile = agent_profile::ActiveModel {
        name: Set("Test Profile".to_string()),
        kill_date: Set(Some(DateTime::new(
            NaiveDate::from_ymd_opt(2050, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        ))),
        working_hours: Set(Some(vec![
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        ])),
        polling_interval: Set(humantime::Duration::from(std::time::Duration::from_mins(1)).to_string()),
        polling_jitter: Set(humantime::Duration::from(std::time::Duration::from_secs(20)).to_string()),
        ..Default::default()
    };
    let profile = new_profile.insert(&db).await.map_err(|e| {
        error!("Failed to create agent profile: {}", e);
        "Failed to create agent profile".to_string()
    })?;

    info!("Test agent profile inserted, id: {}", profile.id);

    // insert filters
    let new_filter = filter::ActiveModel {
        agent_profile_id: Set(profile.id.clone()),
        agent_field: Set(AgentField::Hostname),
        filter_op: Set(FilterOperation::Equals),
        value: Set(serde_json::Value::String("DESKTOP-PC".to_string())),
        sequence: Set(1),
        next_hop_relation: Set(None),
        grouping_start: Set(false),
        grouping_end: Set(false),
        ..Default::default()
    };
    let filter = new_filter.insert(&db).await.map_err(|e| {
        error!("Failed to create filter: {}", e);
        "Failed to create filter".to_string()
    })?;

    info!("Test filter inserted, id: {}", filter.id);

    Ok(())
}

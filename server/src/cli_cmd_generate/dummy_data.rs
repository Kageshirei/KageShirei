//! Dummy data generation for demo purposes

use kageshirei_communication_protocol::{NetworkInterface, NetworkInterfaceArray};
use log::{error, info};
use srv_mod_config::SharedConfig;
use srv_mod_entity::{
    active_enums::{AgentField, AgentIntegrity, FilterOperation},
    entities::{agent, agent_profile, filter, user},
    sea_orm::{
        prelude::*,
        sqlx::types::chrono::{NaiveDate, NaiveTime},
        ActiveValue::Set,
    },
};

use crate::auto_migrate;

/// Generate dummy data for demo purposes
#[expect(
    clippy::module_name_repetitions,
    reason = "The module is generally imported without full classification, the name avoids useless confusion"
)]
pub async fn make_dummy_data(config: SharedConfig) -> Result<(), String> {
    let readonly_config = config.read().await;
    let db = auto_migrate::run(&readonly_config.database.url, &readonly_config).await?;
    drop(readonly_config);

    // Insert the operator into the database
    let new_user = user::ActiveModel {
        username: Set("test".to_owned()),
        password: Set(kageshirei_crypt::hash::argon::Hash::make_password("test").map_err(|e| e.to_string())?),
        ..Default::default()
    };

    new_user.insert(&db).await.map_err(|e| {
        error!("Failed to create operator: {}", e);
        "Failed to create operator".to_owned()
    })?;

    info!("Test user inserted, authenticate using test:test");

    // insert agents
    let new_agent = agent::ActiveModel {
        operating_system: Set("Windows".to_owned()),
        hostname: Set("DESKTOP-PC".to_owned()),
        domain: Set(Some("example-domain".to_owned())),
        username: Set("user".to_owned()),
        network_interfaces: Set(NetworkInterfaceArray {
            network_interfaces: vec![NetworkInterface {
                name:        Some("Ethernet".to_owned()),
                address:     Some("1.2.3.4".to_owned()),
                dhcp_server: Some("10.1.2.3".to_owned()),
            }],
        }),
        pid: Set(1234),
        ppid: Set(5678),
        process_name: Set("example.exe".to_owned()),
        integrity: Set(AgentIntegrity::Medium),
        cwd: Set("C:\\Users\\user".to_owned()),
        server_secret: Set("server-key".to_owned()),
        secret: Set("key".to_owned()),
        signature: Set("signature-0".to_owned()),
        ..Default::default()
    };
    let agent = new_agent.insert(&db).await.map_err(|e| {
        error!("Failed to create agent: {}", e);
        "Failed to create agent".to_owned()
    })?;

    info!("Test agent inserted, id: {}", agent.id);

    let new_profile = agent_profile::ActiveModel {
        name: Set("Test Profile".to_owned()),
        kill_date: Set(Some(DateTime::new(
            NaiveDate::from_ymd_opt(2050, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        ))),
        working_hours: Set(Some(vec![
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        ])),
        polling_interval: Set(humantime::Duration::from(std::time::Duration::from_mins(1)).to_string()),
        polling_jitter: Set(humantime::Duration::from(std::time::Duration::from_secs(20)).to_string()),
        ..Default::default()
    };
    let profile = new_profile.insert(&db).await.map_err(|e| {
        error!("Failed to create agent profile: {}", e);
        "Failed to create agent profile".to_owned()
    })?;

    info!("Test agent profile inserted, id: {}", profile.id);

    // insert filters
    let new_filter = filter::ActiveModel {
        agent_profile_id: Set(profile.id.clone()),
        agent_field: Set(AgentField::Hostname),
        filter_op: Set(FilterOperation::Equals),
        value: Set(serde_json::Value::String("DESKTOP-PC".to_owned())),
        sequence: Set(1),
        next_hop_relation: Set(None),
        grouping_start: Set(false),
        grouping_end: Set(false),
        ..Default::default()
    };
    let filter = new_filter.insert(&db).await.map_err(|e| {
        error!("Failed to create filter: {}", e);
        "Failed to create filter".to_owned()
    })?;

    info!("Test filter inserted, id: {}", filter.id);

    Ok(())
}

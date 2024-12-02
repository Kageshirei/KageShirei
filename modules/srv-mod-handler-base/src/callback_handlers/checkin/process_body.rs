//! Processes the body of a checkin request, this is the entrypoint to the complex checking
//! procedures

use kageshirei_communication_protocol::{
    communication::{Checkin, CheckinResponse},
    Format,
};
use srv_mod_entity::{entities::agent as agent_entity, sea_orm::DatabaseConnection};
use tracing::instrument;

use super::{agent, agent_profiles::apply_filters};
use crate::error;

/// Persist the checkin data into the database as an agent
async fn persist(data: Checkin, db_pool: DatabaseConnection) -> Result<agent_entity::Model, error::CommandHandling> {
    let create_agent_instance = agent::prepare(data)?;

    let db = db_pool.clone();

    agent::create_or_update(create_agent_instance, &db).await
}

#[instrument(skip(db_pool, format))]
pub async fn handle_checkin<F>(
    data: Checkin,
    db_pool: DatabaseConnection,
    mut format: F,
) -> Result<Vec<u8>, error::CommandHandling>
where
    F: Format,
{
    let agent = persist(data, db_pool.clone()).await?;

    // apply filters to the agent
    let config = apply_filters(&agent, db_pool.clone()).await;

    format
        .write::<CheckinResponse, &str>(config, None)
        .map_err(error::CommandHandling::Format)
}

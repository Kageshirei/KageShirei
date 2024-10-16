//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.1

use std::time::Duration;

use sea_orm::{entity::prelude::*, sqlx::types::chrono::Utc, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::helpers::CUID2;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "agent_profile")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub id:               String,
    #[sea_orm(unique)]
    pub name:             String,
    pub kill_date:        Option<DateTime>,
    pub working_hours:    Option<Vec<Time>>,
    pub polling_interval: String,
    pub polling_jitter:   String,
    pub created_at:       DateTime,
    pub updated_at:       DateTime,
}

impl Model {
    /// Check if the working hours are valid
    ///
    /// The working hours are valid if:
    /// - They are in pairs, e.g. 09:00-17:00 **AND**
    /// - Each start time is before the end time
    /// - If the working hours are not set, they are considered valid
    ///
    /// # Returns
    ///
    /// `true` if the working hours are valid, `false` otherwise
    pub fn is_working_hours_valid(&self) -> bool {
        if self.working_hours.is_none() {
            return true;
        }

        let working_hours = self.working_hours.as_ref().unwrap();

        // Check if the working hours are in pairs, e.g. 09:00-17:00
        if working_hours.len() % 2 != 0 {
            return false;
        }

        // Check if the working hours are ordered (each start time is before the end time)
        for i in 0 .. working_hours.len() / 2 {
            let start = working_hours[i * 2];
            let end = working_hours[i * 2 + 1];
            if start >= end {
                return false;
            }
        }

        true
    }

    /// Get the polling interval as a `Duration`
    ///
    /// # Returns
    ///
    /// The polling interval as a `Duration` if it is valid, an error message otherwise
    pub fn get_polling_interval(&self) -> Result<Duration, String> {
        let interval = humantime::parse_duration(&self.polling_interval);
        match interval {
            Ok(interval) => Ok(interval),
            Err(e) => Err(format!("Invalid polling interval: {}", e)),
        }
    }

    /// Get the polling jitter as a `Duration`
    ///
    /// # Returns
    ///
    /// The polling jitter as a `Duration` if it is valid, an error message otherwise
    pub fn get_polling_jitter(&self) -> Result<Duration, String> {
        let jitter = humantime::parse_duration(&self.polling_jitter);
        match jitter {
            Ok(jitter) => Ok(jitter),
            Err(e) => Err(format!("Invalid polling jitter: {}", e)),
        }
    }

    /// Set the polling interval
    ///
    /// # Arguments
    ///
    /// * `interval` - The polling interval
    ///
    /// # Returns
    ///
    /// The updated model
    pub fn set_polling_interval(&mut self, interval: Duration) -> &Self {
        self.polling_interval = humantime::format_duration(interval).to_string();

        self
    }

    /// Set the polling jitter
    ///
    /// # Arguments
    ///
    /// * `jitter` - The polling jitter
    ///
    /// # Returns
    ///
    /// The updated model
    pub fn set_polling_jitter(&mut self, jitter: Duration) -> &Self {
        self.polling_jitter = humantime::format_duration(jitter).to_string();

        self
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::filter::Entity")]
    Filter,
}

impl Related<super::filter::Entity> for Entity {
    fn to() -> RelationDef { Relation::Filter.def() }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            // Generate a new unique ID
            id: Set(CUID2.create_id()),
            ..ActiveModelTrait::default()
        }
    }

    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // Clone the model to avoid moving it
        let mut model = self;

        if insert {
            // Update the `created_at` field with the current time
            model.created_at = Set(Utc::now().naive_utc());
        }

        // Update the `updated_at` field with the current time
        model.updated_at = Set(Utc::now().naive_utc());
        Ok(model)
    }
}

impl ActiveModel {
    /// Check if the working hours are valid
    ///
    /// The working hours are valid if:
    /// - They are in pairs, e.g. 09:00-17:00 **AND**
    /// - Each start time is before the end time
    /// - If the working hours are not set, they are considered valid
    ///
    /// # Returns
    ///
    /// `true` if the working hours are valid, `false` otherwise
    pub fn is_working_hours_valid(&self) -> bool {
        let working_hours = self.working_hours.as_ref();

        if working_hours.is_none() {
            return true;
        }

        let working_hours = working_hours.as_ref().unwrap();

        // Check if the working hours are in pairs, e.g. 09:00-17:00
        if working_hours.len() % 2 != 0 {
            return false;
        }

        // Check if the working hours are ordered (each start time is before the end time)
        for i in 0 .. working_hours.len() / 2 {
            let start = working_hours[i * 2];
            let end = working_hours[i * 2 + 1];
            if start >= end {
                return false;
            }
        }

        true
    }

    /// Get the polling interval as a `Duration`
    ///
    /// # Returns
    ///
    /// The polling interval as a `Duration` if it is valid, an error message otherwise
    pub fn get_polling_interval(&self) -> Result<Duration, String> {
        let interval = humantime::parse_duration(self.polling_interval.as_ref());
        match interval {
            Ok(interval) => Ok(interval),
            Err(e) => Err(format!("Invalid polling interval: {}", e)),
        }
    }

    /// Get the polling jitter as a `Duration`
    ///
    /// # Returns
    ///
    /// The polling jitter as a `Duration` if it is valid, an error message otherwise
    pub fn get_polling_jitter(&self) -> Result<Duration, String> {
        let jitter = humantime::parse_duration(self.polling_jitter.as_ref());
        match jitter {
            Ok(jitter) => Ok(jitter),
            Err(e) => Err(format!("Invalid polling jitter: {}", e)),
        }
    }

    /// Set the polling interval
    ///
    /// # Arguments
    ///
    /// * `interval` - The polling interval
    ///
    /// # Returns
    ///
    /// The updated model
    pub fn set_polling_interval(&mut self, interval: Duration) -> &Self {
        self.polling_interval = Set(humantime::format_duration(interval).to_string());

        self
    }

    /// Set the polling jitter
    ///
    /// # Arguments
    ///
    /// * `jitter` - The polling jitter
    ///
    /// # Returns
    ///
    /// The updated model
    pub fn set_polling_jitter(&mut self, jitter: Duration) -> &Self {
        self.polling_jitter = Set(humantime::format_duration(jitter).to_string());

        self
    }
}

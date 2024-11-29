//! Operator generation command

use log::{error, info};
use srv_mod_config::SharedConfig;
use srv_mod_entity::{
    entities::user,
    sea_orm::{prelude::*, ActiveValue::Set},
};

use crate::{auto_migrate, cli::generate::operator::GenerateOperatorArguments};

/// Generate an operator
///
/// # Parameters
///
/// - `args` - The arguments for the operator generation.
/// - `config` - The shared configuration.
///
/// # Returns
///
/// The result of the operator generation.
#[expect(
    clippy::module_name_repetitions,
    reason = "The module is generally imported without full classification, the name avoids useless confusion"
)]
pub async fn generate_operator(args: &GenerateOperatorArguments, config: SharedConfig) -> Result<(), String> {
    let readonly_config = config.read().await;
    let db = auto_migrate::run(&readonly_config.database.url, &readonly_config).await?;
    drop(readonly_config);

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&args.username))
        .one(&db)
        .await
        .map_err(|e| {
            error!("Failed to check if operator exists: {}", e);
            "Failed to check if operator exists".to_owned()
        })?;

    if user.is_some() {
        error!("Operator with username '{}' already exists", args.username);
        return Err("User already exists".to_owned());
    }

    // Insert the operator into the database
    let new_user = user::ActiveModel {
        username: Set(args.username.clone()),
        password: Set(
            kageshirei_crypt::hash::argon::Hash::make_password(args.password.as_str()).map_err(|e| e.to_string())?,
        ),
        ..Default::default()
    };

    let user = new_user.insert(&db).await.map_err(|e| {
        error!("Failed to create operator: {}", e);
        "Failed to create operator".to_owned()
    })?;

    info!("Created operator with id: {}", user.id);
    Ok(())
}

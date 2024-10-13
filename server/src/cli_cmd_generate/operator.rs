use log::{error, info};
use srv_mod_config::SharedConfig;
use srv_mod_entity::{
    entities::user,
    sea_orm::{prelude::*, ActiveValue::Set},
};

use crate::{auto_migrate, cli::generate::operator::GenerateOperatorArguments};

pub async fn generate_operator(args: &GenerateOperatorArguments, config: SharedConfig) -> Result<(), String> {
    let readonly_config = config.read().await;
    let db = auto_migrate::run(&readonly_config.database.url, &readonly_config).await?;

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&args.username))
        .one(&db)
        .await
        .map_err(|e| {
            error!("Failed to check if operator exists: {}", e);
            "Failed to check if operator exists".to_string()
        })?;

    if user.is_some() {
        error!("Operator with username '{}' already exists", args.username);
        return Err("User already exists".to_string());
    }

    // Insert the operator into the database
    let new_user = user::ActiveModel {
        username: Set(args.username.clone()),
        password: Set(rs2_crypt::argon::Argon2::hash_password(
            args.password.as_str(),
        )?),
        ..Default::default()
    };

    let user = new_user.insert(&db).await.map_err(|e| {
        error!("Failed to create operator: {}", e);
        "Failed to create operator".to_string()
    })?;

    info!("Created operator with id: {}", user.id);
    Ok(())
}

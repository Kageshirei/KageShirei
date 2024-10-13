use log::error;
use validator::{ValidationErrors, ValidationErrorsKind};

/// Print validation errors to the SYNC log
pub fn print_validation_error(validation_errors: ValidationErrors) -> anyhow::Result<()> {
    for (field, errors) in validation_errors.errors() {
        // Errors is an enum with 3 variants: Struct, List, Field
        match errors {
            ValidationErrorsKind::Struct(error) => {
                for err in error.field_errors().iter() {
                    for e in err.1.iter() {
                        error!("{}", parse_field_error(field, e));
                    }
                }
            },
            ValidationErrorsKind::List(error) => {
                for error in error {
                    for err in error.1.field_errors() {
                        for e in err.1.iter() {
                            error!("{}", parse_field_error(field, e));
                        }
                    }
                }
            },
            ValidationErrorsKind::Field(error) => {
                for e in error.iter() {
                    error!("{}", parse_field_error(field, e));
                }
            },
        }
    }

    Ok(())
}

/// Parse a field error into a human-readable string
fn parse_field_error(field: &str, error: &validator::ValidationError) -> String {
    match error.code.to_string().as_str() {
        "__internal__" => {
            format!(
                "Validation error in field '{}': {}",
                field,
                error.message.as_ref().unwrap()
            )
        },
        "range" => {
            format!(
                "Validation error in field '{}': Value '{}' out of the defined range of {}-{}",
                field,
                error.params.get("value").unwrap(),
                error.params.get("min").unwrap_or(
                    error
                        .params
                        .get("exclusive_min")
                        .unwrap_or(&serde_json::Value::String("(Unspecified)".to_string()))
                ),
                error.params.get("max").unwrap_or(
                    error
                        .params
                        .get("exclusive_max")
                        .unwrap_or(&serde_json::Value::String("(Unspecified)".to_string()))
                ),
            )
        },
        "regex" => {
            format!(
                "Validation error in field '{}': {}",
                field,
                error.message.as_ref().unwrap().to_string().replace(
                    ":params.value",
                    error
                        .params
                        .get("value")
                        .unwrap_or(&serde_json::Value::String("Not found".to_string()))
                        .as_str()
                        .unwrap(),
                )
            )
        },
        "length" => {
            let has_min = error.params.get("min").is_some();
            let has_max = error.params.get("max").is_some();
            let has_equal = error.params.get("equal").is_some();

            let mut message = String::new();

            if has_min && !has_max {
                let value = error.params.get("value").unwrap();
                message.push_str(&format!(
                    "A minimum length of {} is required, {} given",
                    error.params.get("min").unwrap(),
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    } else {
                        value.as_str().unwrap().len()
                    }
                ));
            } else if !has_min && has_max {
                let value = error.params.get("value").unwrap();
                message.push_str(&format!(
                    "A maximum length of {} is required, {} given",
                    error.params.get("max").unwrap(),
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    } else {
                        value.as_str().unwrap().len()
                    }
                ));
            } else if has_equal {
                let value = error.params.get("value").unwrap();
                message.push_str(&format!(
                    "An exact length of {} is required, {} given",
                    error.params.get("equal").unwrap(),
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    } else {
                        value.as_str().unwrap().len()
                    }
                ));
            } else {
                let value = error.params.get("value").unwrap();
                message.push_str(&format!(
                    "A length between {} and {} is required, {} given",
                    error.params.get("min").unwrap(),
                    error.params.get("max").unwrap(),
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    } else {
                        value.as_str().unwrap().len()
                    }
                ));
            }

            format!("Validation error in field '{}': {}", field, message)
        },
        _ => {
            format!("Validation error in field '{}': {:?}", field, error)
        },
    }
}

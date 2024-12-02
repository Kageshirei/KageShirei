//! Print validation errors to the log

use std::{
    borrow::ToOwned as _,
    fmt::Write as _,
    format,
    string::{String, ToString as _},
};

use log::error;
use validator::{ValidationErrors, ValidationErrorsKind};

use crate::Configuration;

/// Print validation errors to the SYNC log
pub fn print_validation_error(validation_errors: ValidationErrors) -> Result<(), Configuration> {
    for (field, errors) in validation_errors.errors() {
        // Errors is an enum with 3 variants: Struct, List, Field
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot move out of the enum variant"
        )]
        match errors {
            ValidationErrorsKind::Struct(error) => {
                for err in error.field_errors().iter() {
                    for e in err.1.iter() {
                        error!("{}", parse_field_error(field, e)?);
                    }
                }
            },
            ValidationErrorsKind::List(error) => {
                for error in error {
                    for err in error.1.field_errors() {
                        for e in err.1.iter() {
                            error!("{}", parse_field_error(field, e)?);
                        }
                    }
                }
            },
            ValidationErrorsKind::Field(error) => {
                for e in error.iter() {
                    error!("{}", parse_field_error(field, e)?);
                }
            },
        }
    }

    Ok(())
}

/// Parse a field error into a human-readable string
fn parse_field_error(field: &str, error: &validator::ValidationError) -> Result<String, Configuration> {
    match error.code.to_string().as_str() {
        "__internal__" => {
            Ok(format!(
                "Validation error in field '{}': {}",
                field,
                error.message.as_ref().unwrap()
            ))
        },
        "range" => {
            let value = &error
                .params
                .get("value")
                .ok_or(Configuration::MissingWrongField("value".to_owned()))?;

            Ok(format!(
                "Validation error in field '{}': Value '{}' out of the defined range of {}-{}",
                field,
                value,
                error.params.get("min").unwrap_or(
                    error
                        .params
                        .get("exclusive_min")
                        .unwrap_or(&serde_json::Value::String("(Unspecified)".to_owned()))
                ),
                error.params.get("max").unwrap_or(
                    error
                        .params
                        .get("exclusive_max")
                        .unwrap_or(&serde_json::Value::String("(Unspecified)".to_owned()))
                ),
            ))
        },
        "regex" => {
            Ok(format!(
                "Validation error in field '{}': {}",
                field,
                error.message.as_ref().unwrap().to_string().replace(
                    ":params.value",
                    error
                        .params
                        .get("value")
                        .unwrap_or(&serde_json::Value::String("Not found".to_owned()))
                        .as_str()
                        .unwrap(),
                )
            ))
        },
        "length" => {
            let has_min = error.params.contains_key("min");
            let has_max = error.params.contains_key("max");
            let has_equal = error.params.contains_key("equal");

            let mut message = String::new();

            let value = &error
                .params
                .get("value")
                .ok_or(Configuration::MissingWrongField("value".to_owned()))?;

            if has_min && !has_max {
                write!(
                    message,
                    "A minimum length of '{}' is required, '{}' given",
                    &error
                        .params
                        .get("min")
                        .ok_or(Configuration::MissingValidationLowerBound("min".to_owned()))?,
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    }
                    else {
                        value.as_str().unwrap().len()
                    }
                )
                .map_err(|e| Configuration::Generic(Box::new(e)))?;
            }
            else if !has_min && has_max {
                write!(
                    message,
                    "A maximum length of '{}' is required, '{}' given",
                    &error
                        .params
                        .get("max")
                        .ok_or(Configuration::MissingValidationUpperBound("max".to_owned()))?,
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    }
                    else {
                        value.as_str().unwrap().len()
                    }
                )
                .map_err(|e| Configuration::Generic(Box::new(e)))?;
            }
            else if has_equal {
                write!(
                    message,
                    "An exact length of '{}' is required, '{}' given",
                    &error
                        .params
                        .get("equal")
                        .ok_or(Configuration::MissingEqualField("equal".to_owned()))?,
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    }
                    else {
                        value.as_str().unwrap().len()
                    }
                )
                .map_err(|e| Configuration::Generic(Box::new(e)))?;
            }
            else {
                write!(
                    message,
                    "A length between '{}' and '{}' is required, '{}' given",
                    &error
                        .params
                        .get("min")
                        .ok_or(Configuration::MissingValidationLowerBound("min".to_owned()))?,
                    &error
                        .params
                        .get("max")
                        .ok_or(Configuration::MissingValidationLowerBound("max".to_owned()))?,
                    if value.is_array() {
                        value.as_array().unwrap().len()
                    }
                    else {
                        value.as_str().unwrap().len()
                    }
                )
                .map_err(|e| Configuration::Generic(Box::new(e)))?;
            }

            Ok(format!(
                "Validation error in field '{}': {}",
                field, message
            ))
        },
        _ => {
            Ok(format!(
                "Validation error in field '{}': {:?}",
                field, error
            ))
        },
    }
}

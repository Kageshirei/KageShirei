use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

pub enum Configuration {
    /// The configuration is validating a struct or field requiring a value to be defined between a
    /// lower and upper bound, the lower one is missing
    MissingValidationLowerBound(String),
    /// The configuration is validating a struct or field requiring a value to be defined between a
    /// lower and upper bound, the upper one is missing
    MissingValidationUpperBound(String),
    /// The configuration is validating a struct or field requiring a value to be defined between a
    /// lower and upper bound, but the field value is missing
    MissingWrongField(String),
    /// The configuration is validating a struct or field requiring a value to be defined as equal
    /// to a specific value, but the equal value is missing
    MissingEqualField(String),
}

impl Debug for Configuration {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Delegate to Display
        write!(f, "{}", self)
    }
}

impl Display for Configuration {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Cannot dereference into the Display trait implementation"
        )]
        match self {
            Configuration::MissingValidationLowerBound(field) => {
                write!(f, "Missing lower bound for field '{}'", field)
            },
            Configuration::MissingValidationUpperBound(field) => {
                write!(f, "Missing upper bound for field '{}'", field)
            },
            Configuration::MissingWrongField(field) => {
                write!(f, "Missing value for field '{}'", field)
            },
        }
    }
}

impl Error for Configuration {}

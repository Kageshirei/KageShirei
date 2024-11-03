//! Helper functions for the entity module.

use cuid2::CuidConstructor;
use once_cell::sync::Lazy;

/// A CUID generator with a length of 32 characters, used for generating unique identifiers.
pub static CUID2: Lazy<CuidConstructor> = Lazy::new(|| {
    let mut cuid2 = CuidConstructor::new();
    cuid2.set_length(32);
    cuid2
});

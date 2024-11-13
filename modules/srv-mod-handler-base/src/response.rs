//! The response returned by the base handler

use std::num::NonZeroU16;

use srv_mod_config::handlers;

/// The response returned by the base handler
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::module_name_repetitions,
    reason = "The name is descriptive and will be used in higher order modules"
)]
pub struct BaseHandlerResponse {
    /// The status code of the response
    pub status:    NonZeroU16,
    /// The raw body of the response
    pub body:      Vec<u8>,
    /// The formatter instance that should be used to format the response if any
    pub formatter: Option<handlers::Format>,
}

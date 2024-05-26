use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub request_id: Uuid,
    pub command_id: Uuid,
    pub path: Option<String>,
}

/// Define the metadata trait responsible for providing metadata about a type.
pub trait WithMetadata {
    /// Get the metadata for the type.
    fn get_metadata(&self) -> Metadata;
}
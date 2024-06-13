use serde::{Deserialize, Serialize};

/// The result of a command requiring additional post-processing in the terminal emulator.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostProcessResult<T>
where
    T: Serialize,
{
    /// The type of the data.
    pub r#type: String,
    /// The data to be processed.
    pub data: T,
}
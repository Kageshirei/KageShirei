/// Utility error for server components to use when an unrecoverable error is detected and shutdown is required.
pub fn unrecoverable_error() -> anyhow::Result<()> { Err(anyhow::anyhow!("Unrecoverable error(s) detected, exiting.")) }

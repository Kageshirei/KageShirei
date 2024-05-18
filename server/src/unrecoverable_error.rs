/// Utility error
pub fn unrecoverable_error() -> anyhow::Result<()> {
    Err(anyhow::anyhow!("Unrecoverable error(s) detected, exiting."))
}
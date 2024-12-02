use std::time::Duration;

pub trait DurationExt {
    fn round(&self) -> Duration;
}

impl DurationExt for Duration {
    /// Truncate the duration to the millisecond
    fn round(&self) -> Duration {
        let millis = self.as_millis() + self.subsec_millis() as u128;

        // Convert the truncated nanoseconds back to a Duration
        Duration::from_millis(millis as u64)
    }
}

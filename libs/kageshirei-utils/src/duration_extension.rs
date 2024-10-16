use core::time::Duration;

pub trait DurationExt {
    fn round(&self) -> Duration;
}

impl DurationExt for Duration {
    /// Truncate the duration to the millisecond
    fn round(&self) -> Duration {
        let millis = (self.as_millis() as u64).saturating_add(self.subsec_millis() as u64);

        // Convert the truncated nanoseconds back to a Duration
        Self::from_millis(millis)
    }
}

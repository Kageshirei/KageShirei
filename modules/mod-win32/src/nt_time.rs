use mod_agentcore::instance;
use rs2_win32::ntdef::LargeInteger;

/// Gets the current timestamp in Unix format.
///
/// This function reads the Windows SharedUserData to retrieve the current
/// system time in 100-nanosecond intervals since January 1, 1601 (UTC) and converts
/// it to Unix epoch time (seconds since January 1, 1970).
///
/// The function ensures a consistent read by repeatedly reading the high and low parts
/// of the system time until the values are stable. This approach mimics the behavior of
/// the Windows API function `GetSystemTimeAsFileTime`.
///
/// Returns:
///     The current timestamp in Unix format (i64).
pub fn current_timestamp() -> i64 {
    unsafe {
        let mut system_time: LargeInteger = LargeInteger::new();

        // Loop to ensure consistent read of system time
        // The loop reads high1_time and low_part, and checks if high1_time is equal to high2_time
        // to confirm that the values are stable.
        loop {
            system_time.high_part = (*instance().kdata).system_time.high1_time;
            system_time.low_part = (*instance().kdata).system_time.low_part;
            if system_time.high_part == (*instance().kdata).system_time.high2_time {
                break;
            }
        }

        // Combine high_part and low_part into a single 64-bit integer
        // high_part is shifted left by 32 bits to make space for low_part
        let high_part: u64 = system_time.high_part as i64 as u64;
        let low_part: u64 = system_time.low_part as u64;
        let system_time_100ns: u64 = (high_part << 32) | low_part;

        // Convert the system time from 100-nanosecond intervals since January 1, 1601 (UTC)
        // to seconds since January 1, 1970 (Unix epoch time).
        // 11644473600 seconds is the difference between the two epochs.
        let unix_epoch_time: i64 = (system_time_100ns / 10_000_000) as i64 - 11_644_473_600;

        unix_epoch_time
    }
}

/// Checks the provided kill date against the current timestamp and exits if the current time exceeds the kill date.
///
/// Args:
///     opt_timestamp (Option<i64>): The optional Unix timestamp representing the kill date.
///
/// Returns:
///     A boolean indicating if the current time exceeds the kill date.
pub fn check_kill_date(opt_timestamp: Option<i64>) -> bool {
    if let Some(kill_date) = opt_timestamp {
        let current_time = current_timestamp();
        if current_time > kill_date {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use libc_print::libc_println;

    #[test]
    fn test_current_timestamp() {
        let timestamp = current_timestamp();
        libc_println!("Current timestamp: {}", timestamp);

        // Check if the timestamp is a reasonable value (greater than the Unix epoch start)
        assert!(timestamp > 0);

        // Convert the timestamp to a readable format
        let datetime = DateTime::from_timestamp(timestamp, 0);
        libc_println!("Current datetime: {}", datetime.unwrap());
    }

    #[test]
    fn test_check_kill_date() {
        // Set a kill date in the future
        let future_kill_date = current_timestamp() + 10000;
        libc_println!("Future kill date: {}", future_kill_date);

        // This should not cause an exit
        let should_exit = check_kill_date(Some(future_kill_date));
        let future_datetime = DateTime::from_timestamp(future_kill_date, 0);
        libc_println!("Future datetime: {}", future_datetime.unwrap());
        assert!(!should_exit);

        // Set a kill date in the past
        let past_kill_date = current_timestamp() - 10000;
        libc_println!("Past kill date: {}", past_kill_date);

        let past_datetime = DateTime::from_timestamp(past_kill_date, 0);
        libc_println!("Past datetime: {}", past_datetime.unwrap());

        // This should print "Exit..."
        let should_exit = check_kill_date(Some(past_kill_date));
        if should_exit {
            libc_println!("Exit...");
        }
        assert!(should_exit);
    }
}

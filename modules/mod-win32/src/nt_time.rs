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

/// Delays the execution of the current thread for the specified duration in seconds.
///
/// This function wraps the `NtDelayExecution` syscall to pause the thread for the provided
/// duration. The delay is not alertable, meaning it cannot be interrupted by asynchronous events.
///
/// Args:
///     seconds (i64): The duration to delay in seconds.
pub fn delay(seconds: i64) {
    // Convert seconds to 100-nanosecond intervals
    let delay_interval = -seconds * 10_000_000;

    // Call NtDelayExecution
    unsafe {
        instance()
            .ntdll
            .nt_delay_execution
            .run(false, &delay_interval);
    }
}

/// Converts a Unix timestamp (seconds since January 1, 1970) to a human-readable date and time format.
///
/// The function returns a tuple representing the date and time in the format (year, month, day, hour, minute, second).
///
/// # Arguments
///
/// * `timestamp` - A Unix timestamp as an `i64`.
///
/// # Returns
///
/// * A tuple (year, month, day, hour, minute, second) representing the corresponding date and time.
pub fn timestamp_to_datetime(timestamp: i64) -> (i64, u8, u8, u8, u8, u8) {
    // Number of days since Unix epoch (January 1, 1970)
    let days = timestamp / 86_400;
    let mut seconds = timestamp % 86_400;

    // Calculate hour, minute, and second
    let hour = (seconds / 3_600) as u8;
    seconds %= 3_600;
    let minute = (seconds / 60) as u8;
    let second = (seconds % 60) as u8;

    // Calculate year, month, and day
    let mut year = 1970;
    let mut remaining_days = days;

    while remaining_days >= days_in_year(year) {
        remaining_days -= days_in_year(year);
        year += 1;
    }

    let mut month = 0;
    while remaining_days >= days_in_month(year, month) {
        remaining_days -= days_in_month(year, month);
        month += 1;
    }

    let day = remaining_days + 1; // day starts from 1

    (year, (month + 1) as u8, day as u8, hour, minute, second)
}

/// Helper function to determine if a year is a leap year.
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Helper function to return the number of days in a year.
fn days_in_year(year: i64) -> i64 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

/// Helper function to return the number of days in a specific month of a year.
fn days_in_month(year: i64, month: usize) -> i64 {
    match month {
        0 => 31, // January
        1 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        } // February
        2 => 31, // March
        3 => 30, // April
        4 => 31, // May
        5 => 30, // June
        6 => 31, // July
        7 => 31, // August
        8 => 30, // September
        9 => 31, // October
        10 => 30, // November
        11 => 31, // December
        _ => 0,  // Invalid month
    }
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
    fn test_timestamp_to_datetime() {
        let timestamp = current_timestamp();
        libc_println!("Current timestamp: {}", timestamp);

        let datetime = timestamp_to_datetime(timestamp);
        libc_println!(
            "Current datetime: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            datetime.0,
            datetime.1,
            datetime.2,
            datetime.3,
            datetime.4,
            datetime.5
        );

        // Basic assertion to check if year is reasonable
        assert!(datetime.0 >= 1970);
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

    #[test]
    fn test_delay_execution() {
        libc_println!("Starting delay...");

        // Esegui un ritardo di 2 secondi
        delay(2);

        libc_println!("Delay of 2 seconds completed");

        // Esegui un ritardo di 5 secondi
        delay(15);

        libc_println!("Delay of 5 seconds completed");
    }
}

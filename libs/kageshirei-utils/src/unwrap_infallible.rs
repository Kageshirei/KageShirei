use core::convert::Infallible;

/// Unwrap a result that is guaranteed to never be an error.
pub fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}

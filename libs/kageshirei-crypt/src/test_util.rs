#[cfg(test)]
#[macro_export]
macro_rules! no_std_println {
    ($fmt:expr $(, $args:expr)*) => {{
        unsafe {
            // Format the string using the format! macro
            let formatted = format!(concat!($fmt, "\n\0"), $($args),*);

            // Convert it to a null-terminated C string
            let c_string = core::ffi::CStr::from_bytes_with_nul(formatted.as_bytes()).unwrap();

            // Call printf with the C string pointer
            libc::printf(c_string.as_ptr());
        }
    }};
}

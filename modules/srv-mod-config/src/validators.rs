use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

pub static IP_V4_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|localhost)$").unwrap());

/// Validate that the port is within the valid range
pub fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port < 1024 {
        #[cfg(unix)]
        {
            // On Unix systems, check if the user is root (uid 0)
            if !nix::unistd::Uid::effective().is_root() {
                return Err(ValidationError::new("__internal__").with_message(Cow::from(
                    "Ports below 1024 require root privileges to bind",
                )))
            }
        }
        #[cfg(windows)]
        {
            // On Windows, check if the process has administrative privileges
            use std::ptr;
            use winapi::shared::minwindef::DWORD;
            use winapi::um::handleapi::CloseHandle;
            use winapi::um::processthreadsapi::OpenProcessToken;
            use winapi::um::securitybaseapi::GetTokenInformation;
            use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};

            unsafe {
                let mut token_handle = ptr::null_mut();
                if OpenProcessToken(
                    winapi::um::processthreadsapi::GetCurrentProcess(),
                    TOKEN_QUERY,
                    &mut token_handle,
                ) != 0
                {
                    let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
                    let mut length = size_of::<TOKEN_ELEVATION>() as DWORD;

                    if GetTokenInformation(
                        token_handle,
                        TokenElevation,
                        &mut elevation as *mut _ as *mut _,
                        length,
                        &mut length,
                    ) != 0
                    {
                        if elevation.TokenIsElevated != 0 {
                            return Ok(()); // User has administrative privileges
                        }
                    }
                }

                // If the process is not elevated, return an error
                return Err(ValidationError::new("__internal__").with_message(Cow::from(
                    "Ports below 1024 require administrative privileges on Windows",
                )))
            }
        }
    }
    // Port is within the valid range
    Ok(())
}

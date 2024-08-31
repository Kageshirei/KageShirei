use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use rs2_win32::{ntdef::UnicodeString, ntstatus::*};

/// Converts a Rust string slice to a wide string (Vec<u16>) with a null terminator.
///
/// This function takes a string slice and converts it to a vector of 16-bit Unicode
/// (UTF-16) code units, appending a null terminator at the end. This is useful for
/// interacting with Windows API functions that expect wide strings (LPCWSTR).
///
/// # Arguments
/// * `s` - A string slice to be converted.
///
/// # Returns
/// A `Vec<u16>` containing the UTF-16 encoded representation of the input string,
/// followed by a null terminator.
pub fn to_pcwstr(s: &str) -> Vec<u16> {
    // Encode the input string as UTF-16 and append a null terminator (0).
    s.encode_utf16().chain(Some(0)).collect()
}

/// Represents the result of parsing a URL.
pub struct ParseUrlResult {
    /// The scheme of the URL (0x01 for HTTP, 0x02 for HTTPS).
    pub scheme: u16,
    /// The hostname extracted from the URL.
    pub hostname: String,
    /// The port number extracted from the URL.
    pub port: u16,
    /// The path extracted from the URL.
    pub path: String,
}

impl ParseUrlResult {
    /// Creates a new `ParseUrlResult`.
    ///
    /// This method initializes a new instance of the `ParseUrlResult` struct with the provided values.
    ///
    /// # Arguments
    /// * `scheme` - The scheme of the URL (0x01 for HTTP, 0x02 for HTTPS).
    /// * `hostname` - The hostname extracted from the URL.
    /// * `port` - The port number extracted from the URL.
    /// * `path` - The path extracted from the URL.
    ///
    /// # Returns
    /// A new `ParseUrlResult` instance with the provided scheme, hostname, port, and path.
    pub fn new(scheme: u16, hostname: String, port: u16, path: String) -> Self {
        ParseUrlResult {
            scheme,
            hostname,
            port,
            path,
        }
    }
}

/// Extracts the scheme, hostname, port, and path from a URL.
///
/// This function takes a URL with or without the scheme (http:// or https://) and returns the scheme,
/// hostname, appropriate port number, and the path.
///
/// # Arguments
/// * `url` - The full URL with the scheme.
///
/// # Returns
/// A tuple containing:
/// * `u16` - The scheme (0x01 for HTTP, 0x02 for HTTPS).
/// * `String` - The hostname.
/// * `u16` - The port number (default 80 for HTTP and 443 for HTTPS if not specified).
/// * `String` - The path.
pub fn parse_url(url: &str) -> ParseUrlResult {
    let (scheme, rest) = if url.starts_with("http://") {
        (0x01, url.trim_start_matches("http://"))
    } else if url.starts_with("https://") {
        (0x02, url.trim_start_matches("https://"))
    } else {
        (0x01, url)
    };

    let (hostname, port, path) = if let Some(pos) = rest.find(':') {
        let (host, port_and_path) = rest.split_at(pos);
        let mut parts = port_and_path.splitn(2, '/');
        let port_str = parts.next().unwrap().trim_start_matches(':');
        let port = port_str
            .parse()
            .unwrap_or(if scheme == 0x01 { 80 } else { 443 });
        let path = parts.next().unwrap_or("");
        (host.to_string(), port, format!("/{}", path))
    } else if let Some(pos) = rest.find('/') {
        let (host, path) = rest.split_at(pos);
        (
            host.to_string(),
            if scheme == 0x01 { 80 } else { 443 },
            path.to_string(),
        )
    } else {
        (
            rest.to_string(),
            if scheme == 0x01 { 80 } else { 443 },
            "/".to_string(),
        )
    };

    ParseUrlResult::new(scheme, hostname, port, path)
}

/// Converts a given `UnicodeString` to a Rust `String`.
///
/// This function takes a `UnicodeString` and converts it into an `Option<String>`.
/// If the `UnicodeString` is empty or its buffer is null, it returns `None`.
///
/// # Safety
/// This function performs unsafe operations, such as dereferencing raw pointers.
/// Ensure that the input `UnicodeString` is valid and properly initialized.
///
/// # Parameters
/// - `unicode_string`: A reference to the `UnicodeString` that needs to be converted.
///
/// # Returns
/// An `Option<String>` containing the converted string, or `None` if the conversion fails.
pub fn unicodestring_to_string(unicode_string: &UnicodeString) -> Option<String> {
    // Check if the length of the UnicodeString is zero or if the buffer is null.
    if unicode_string.length == 0 || unicode_string.buffer.is_null() {
        return None;
    }

    // Convert the raw UTF-16 buffer into a Rust slice.
    let slice = unsafe {
        core::slice::from_raw_parts(unicode_string.buffer, (unicode_string.length / 2) as usize)
    };

    // Attempt to convert the UTF-16 slice into a Rust String.
    String::from_utf16(slice).ok()
}

#[allow(non_snake_case)]
pub fn NT_STATUS(status: i32) -> String {
    match status {
        STATUS_SUCCESS => format!("STATUS_SUCCESS [0x{:08X}]", status),
        STATUS_BUFFER_OVERFLOW => format!("STATUS_BUFFER_OVERFLOW [0x{:08X}]", status),
        STATUS_BUFFER_TOO_SMALL => format!("STATUS_BUFFER_TOO_SMALL [0x{:08X}]", status),
        STATUS_OBJECT_NAME_NOT_FOUND => format!("STATUS_OBJECT_NAME_NOT_FOUND [0x{:08X}]", status),
        STATUS_INFO_LENGTH_MISMATCH => format!("STATUS_INFO_LENGTH_MISMATCH [0x{:08X}]", status),
        STATUS_ACCESS_VIOLATION => format!("STATUS_ACCESS_VIOLATION [0x{:08X}]", status),
        STATUS_ACCESS_DENIED => format!("STATUS_ACCESS_DENIED [0x{:08X}]", status),
        STATUS_INVALID_HANDLE => format!("STATUS_INVALID_HANDLE [0x{:08X}]", status),
        STATUS_INSUFFICIENT_RESOURCES => {
            format!("STATUS_INSUFFICIENT_RESOURCES [0x{:08X}]", status)
        }
        STATUS_NOT_IMPLEMENTED => format!("STATUS_NOT_IMPLEMENTED [0x{:08X}]", status),
        STATUS_INVALID_PARAMETER => format!("STATUS_INVALID_PARAMETER [0x{:08X}]", status),
        STATUS_CONFLICTING_ADDRESSES => format!("STATUS_CONFLICTING_ADDRESSES [0x{:08X}]", status),
        STATUS_PRIVILEGE_NOT_HELD => format!("STATUS_PRIVILEGE_NOT_HELD [0x{:08X}]", status),
        STATUS_MEMORY_NOT_ALLOCATED => format!("STATUS_MEMORY_NOT_ALLOCATED [0x{:08X}]", status),
        STATUS_INVALID_PAGE_PROTECTION => {
            format!("STATUS_INVALID_PAGE_PROTECTION [0x{:08X}]", status)
        }
        STATUS_ILLEGAL_INSTRUCTION => format!("STATUS_ILLEGAL_INSTRUCTION [0x{:08X}]", status),
        STATUS_INTEGER_DIVIDE_BY_ZERO => {
            format!("STATUS_INTEGER_DIVIDE_BY_ZERO [0x{:08X}]", status)
        }
        STATUS_DLL_NOT_FOUND => format!("STATUS_DLL_NOT_FOUND [0x{:08X}]", status),
        STATUS_DLL_INIT_FAILED => format!("STATUS_DLL_INIT_FAILED [0x{:08X}]", status),
        STATUS_NO_SUCH_FILE => format!("STATUS_NO_SUCH_FILE [0x{:08X}]", status),
        STATUS_INVALID_DEVICE_REQUEST => {
            format!("STATUS_INVALID_DEVICE_REQUEST [0x{:08X}]", status)
        }
        STATUS_NOT_FOUND => format!("STATUS_NOT_FOUND [0x{:08X}]", status),
        STATUS_DATATYPE_MISALIGNMENT => format!("STATUS_DATATYPE_MISALIGNMENT [0x{:08X}]", status),
        _ => format!("STATUS_UNKNOWN [0x{:08X}]", status),
    }
}

/// Formats a named pipe string and stores it in a `Vec<u16>`.
///
/// This function generates a named pipe path in the format:
/// `\\Device\\NamedPipe\\Win32Pipes.<process_id>.<pipe_id>`
/// and stores the UTF-16 encoded string in a `Vec<u16>`.
///
/// # Parameters
/// - `process_id`: The process ID to be included in the pipe name.
/// - `pipe_id`: The pipe ID to be included in the pipe name.
///
/// # Returns
/// A `Vec<u16>` containing the UTF-16 encoded string, null-terminated.
pub fn format_named_pipe_string(process_id: usize, pipe_id: u32) -> Vec<u16> {
    // Use `format!` to create the pipe name as a String
    let pipe_name = format!(
        "\\Device\\NamedPipe\\Win32Pipes.{:016x}.{:08x}",
        process_id, pipe_id
    );

    // Convert the formatted string into a UTF-16 encoded vector
    let mut pipe_name_utf16: Vec<u16> = pipe_name.encode_utf16().collect();

    // Null-terminate the buffer by pushing a 0 at the end
    pipe_name_utf16.push(0);

    // Return the UTF-16 encoded vector with a null terminator
    pipe_name_utf16
}

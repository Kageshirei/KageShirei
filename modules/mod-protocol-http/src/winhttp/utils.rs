//! Utility functions for the WinHTTP module.

use alloc::{borrow::ToOwned as _, format, string::String, vec::Vec};

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

/// A structure representing the result of parsing a URL.
pub struct ParseUrlResult {
    // 0x01 = http or 0x02 = https
    /// The scheme of the URL.
    pub scheme:   u16,
    /// The hostname.
    pub hostname: String,
    /// The port number.
    pub port:     u16,
    /// The path of the URL.
    pub path:     String,
}

impl ParseUrlResult {
    /// Create a new `ParseUrlResult`.
    pub const fn new(scheme: u16, hostname: String, port: u16, path: String) -> Self {
        Self {
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
    }
    else if url.starts_with("https://") {
        (0x02, url.trim_start_matches("https://"))
    }
    else {
        (0x01, url)
    };

    let (hostname, port, path) = rest.find(':').map_or_else(
        || {
            rest.find('/').map_or_else(
                || {
                    (
                        rest.to_owned(),
                        if scheme == 0x01 { 80 } else { 443 },
                        "/".to_owned(),
                    )
                },
                |pos| {
                    let (host, path) = rest.split_at(pos);
                    (
                        host.to_owned(),
                        if scheme == 0x01 { 80 } else { 443 },
                        path.to_owned(),
                    )
                },
            )
        },
        |pos| {
            let (host, port_and_path) = rest.split_at(pos);
            let mut parts = port_and_path.splitn(2, '/');
            let port_str = parts.next().unwrap().trim_start_matches(':');
            let port = port_str
                .parse()
                .unwrap_or(if scheme == 0x01 { 80 } else { 443 });
            let path = parts.next().unwrap_or("");
            (host.to_owned(), port, format!("/{}", path))
        },
    );

    ParseUrlResult::new(scheme, hostname, port, path)
}

// #[cfg(test)]
// mod tests {
// use super::*;
//
// #[test]
// fn test_parse_url() {
// let parsed_url: ParseUrlResult = parse_url("http://localhost:8080/path");
// assert_eq!(parsed_url.scheme, 0x01);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 8080);
// assert_eq!(parsed_url.path, "/path");
//
// let parsed_url: ParseUrlResult = parse_url("https://localhost:444/another/path");
// assert_eq!(parsed_url.scheme, 0x02);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 444);
// assert_eq!(parsed_url.path, "/another/path");
//
// let parsed_url: ParseUrlResult = parse_url("localhost");
// assert_eq!(parsed_url.scheme, 0x01);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 80);
// assert_eq!(parsed_url.path, "/");
//
// let parsed_url: ParseUrlResult = parse_url("https://localhost");
// assert_eq!(parsed_url.scheme, 0x02);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 443);
// assert_eq!(parsed_url.path, "/");
//
// let parsed_url: ParseUrlResult = parse_url("http://localhost");
// assert_eq!(parsed_url.scheme, 0x01);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 80);
// assert_eq!(parsed_url.path, "/");
//
// let parsed_url: ParseUrlResult = parse_url("https://localhost/");
// assert_eq!(parsed_url.scheme, 0x02);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 443);
// assert_eq!(parsed_url.path, "/");
//
// let parsed_url: ParseUrlResult = parse_url("http://localhost:8080/");
// assert_eq!(parsed_url.scheme, 0x01);
// assert_eq!(parsed_url.hostname, "localhost");
// assert_eq!(parsed_url.port, 8080);
// assert_eq!(parsed_url.path, "/");
// }
// }

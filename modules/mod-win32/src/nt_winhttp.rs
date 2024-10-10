use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::transmute;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

use mod_agentcore::instance;
use mod_agentcore::ldr::ldr_function_addr;
use rs2_win32::ntdef::{GetLastError, UnicodeString};
use rs2_win32::winhttp::{
    WinHttp, WinHttpError, HTTP_QUERY_STATUS_CODE, WINHTTP_ACCESS_TYPE_NO_PROXY,
    WINHTTP_FLAG_BYPASS_PROXY_CACHE, WINHTTP_FLAG_SECURE, WINHTTP_QUERY_FLAG_NUMBER,
};

use crate::utils::{parse_url, to_pcwstr};

// Global variable to store WinHTTP functions
static mut WINHTTP_FUNCS: Option<WinHttp> = None;
// Atomic variable to track if WinHTTP functions have been initialized
static INIT_WINHTTP: AtomicBool = AtomicBool::new(false);

/// Initializes WinHTTP functions.
///
/// This function dynamically loads the WinHTTP functions from the "winhttp.dll" library.
/// It ensures that the functions are loaded only once using atomic operations.
/// The function does not return any values.
fn init_winhttp_funcs() {
    unsafe {
        if !INIT_WINHTTP.load(Ordering::Acquire) {
            pub const WINHTTP_OPEN_DBJ2: usize = 0x613eace5;
            pub const WINHTTP_CONNECT_DBJ2: usize = 0x81e0c81d;
            pub const WINHTTP_OPEN_REQUEST_DBJ2: usize = 0xb06d900e;
            pub const WINHTTP_SET_OPTION_DBJ2: usize = 0x5b6ad378;
            pub const WINHTTP_CLOSE_HANDLE_DBJ2: usize = 0xa7355f15;
            pub const WINHTTP_SEND_REQUEST_DBJ2: usize = 0x7739d0e6;
            pub const WINHTTP_ADD_REQUEST_HEADERS_DBJ2: usize = 0xa2c0b0e1;
            pub const WINHTTP_RECEIVE_RESPONSE_DBJ2: usize = 0xae351ae5;
            pub const WINHTTP_READ_DATA_DBJ2: usize = 0x75064b89;
            pub const WINHTTP_QUERY_HEADERS_DBJ2: usize = 0xcc1a89c5;
            pub const WINHTTP_GET_IE_PROXY_CONFIG_FOR_CURRENT_USER_DBJ2: usize = 0x028197a2;
            pub const WINHTTP_GET_PROXY_FOR_URL_DBJ2: usize = 0xa2cf3c6f;

            //TODO: remove hardcoded dll name
            let dll_name = "winhttp.dll";
            let mut winhttp_dll_unicode = UnicodeString::new();
            let utf16_string: Vec<u16> = dll_name.encode_utf16().chain(Some(0)).collect();
            winhttp_dll_unicode.init(utf16_string.as_ptr());

            let mut winhttp_handle: *mut c_void = null_mut();

            (instance().ntdll.ldr_load_dll)(
                null_mut(),
                null_mut(),
                winhttp_dll_unicode,
                &mut winhttp_handle as *mut _ as *mut c_void,
            );

            if winhttp_handle.is_null() {
                return;
            }

            let h_module = winhttp_handle as *mut u8;

            if h_module.is_null() {
                panic!("Failed to load winhttp.dll");
            }

            // Load function addresses from the module
            let win_http_open_addr = ldr_function_addr(h_module, WINHTTP_OPEN_DBJ2);
            let win_http_connect_addr = ldr_function_addr(h_module, WINHTTP_CONNECT_DBJ2);
            let win_http_open_request_addr = ldr_function_addr(h_module, WINHTTP_OPEN_REQUEST_DBJ2);
            let win_http_set_option_addr = ldr_function_addr(h_module, WINHTTP_SET_OPTION_DBJ2);
            let win_http_close_handle_addr = ldr_function_addr(h_module, WINHTTP_CLOSE_HANDLE_DBJ2);
            let win_http_send_request_addr = ldr_function_addr(h_module, WINHTTP_SEND_REQUEST_DBJ2);
            let win_http_add_request_headers_addr =
                ldr_function_addr(h_module, WINHTTP_ADD_REQUEST_HEADERS_DBJ2);
            let win_http_receive_response_addr =
                ldr_function_addr(h_module, WINHTTP_RECEIVE_RESPONSE_DBJ2);
            let win_http_read_data_addr = ldr_function_addr(h_module, WINHTTP_READ_DATA_DBJ2);
            let win_http_query_headers_addr =
                ldr_function_addr(h_module, WINHTTP_QUERY_HEADERS_DBJ2);
            let win_http_get_ie_proxy_config_addr =
                ldr_function_addr(h_module, WINHTTP_GET_IE_PROXY_CONFIG_FOR_CURRENT_USER_DBJ2);
            let win_http_get_proxy_for_url_addr =
                ldr_function_addr(h_module, WINHTTP_GET_PROXY_FOR_URL_DBJ2);

            let mut winhttp_functions = WinHttp::new();

            // Transmute function addresses to their respective types
            winhttp_functions.win_http_open = transmute(win_http_open_addr);
            winhttp_functions.win_http_connect = transmute(win_http_connect_addr);
            winhttp_functions.win_http_open_request = transmute(win_http_open_request_addr);
            winhttp_functions.win_http_set_option = transmute(win_http_set_option_addr);
            winhttp_functions.win_http_close_handle = transmute(win_http_close_handle_addr);
            winhttp_functions.win_http_send_request = transmute(win_http_send_request_addr);
            winhttp_functions.win_http_add_request_headers =
                transmute(win_http_add_request_headers_addr);
            winhttp_functions.win_http_receive_response = transmute(win_http_receive_response_addr);
            winhttp_functions.win_http_read_data = transmute(win_http_read_data_addr);
            winhttp_functions.win_http_query_headers = transmute(win_http_query_headers_addr);
            winhttp_functions.win_http_get_ie_proxy_config_for_current_user =
                transmute(win_http_get_ie_proxy_config_addr);
            winhttp_functions.win_http_get_proxy_for_url =
                transmute(win_http_get_proxy_for_url_addr);

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
pub fn get_winhttp() -> &'static WinHttp {
    init_winhttp_funcs();
    return unsafe { WINHTTP_FUNCS.as_ref().unwrap() };
}

/// Represents an HTTP response.
pub struct Response {
    /// The status code of the HTTP response.
    pub status_code: u32,
    /// The body of the HTTP response as a string.
    pub body: String,
}

/// Reads the response from an HTTP request.
///
/// This function reads the response from the specified HTTP request handle and
/// returns a `Result` containing the response or an error message if the operation failed.
///
/// # Arguments
/// * `h_request` - The HTTP request handle.
///
/// # Returns
/// * `Ok(Response)` with the response if the operation was successful.
/// * `Err(String)` if there was an error during the read operation, with the error message.
unsafe fn read_response(h_request: *mut c_void) -> Result<Response, String> {
    let mut status_code: u32 = 0;
    let mut status_code_len: u32 = core::mem::size_of::<u32>() as u32;
    let b_status_code = (get_winhttp().win_http_query_headers)(
        h_request,
        HTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
        null(),
        &mut status_code as *mut _ as *mut _,
        &mut status_code_len,
        null_mut(),
    );
    if b_status_code == 0 {
        let error = GetLastError();
        return Err(format!(
            "WinHttpQueryHeaders failed with error: {}",
            WinHttpError::from_code(error as i32)
        ));
    }

    let mut buffer: [u8; 4096] = [0; 4096];
    let mut bytes_read: u32 = 0;
    let mut response_body = String::new();
    loop {
        let b_result = (get_winhttp().win_http_read_data)(
            h_request,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut bytes_read,
        );
        if b_result == 0 || bytes_read == 0 {
            break;
        }
        response_body.push_str(&String::from_utf8_lossy(&buffer[..bytes_read as usize]));
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
fn http_get(url: &str, path: &str) -> Result<Response, String> {
    unsafe {
        let h_session = (get_winhttp().win_http_open)(
            to_pcwstr("RustWinHttp").as_ptr(),
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            null(),
            null(),
            0,
        );
        if h_session.is_null() {
            let error = GetLastError();
            return Err(format!(
                "WinHttpOpen failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let parsed_url = parse_url(url);

        let secure_flag = if parsed_url.scheme == 0x02 {
            WINHTTP_FLAG_SECURE
        } else {
            0
        };

        let h_connect = (get_winhttp().win_http_connect)(
            h_session,
            to_pcwstr(parsed_url.hostname.as_str()).as_ptr(),
            parsed_url.port,
            0,
        );

        if h_connect.is_null() {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpConnect failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let h_request = (get_winhttp().win_http_open_request)(
            h_connect,
            to_pcwstr("GET").as_ptr(),
            to_pcwstr(path).as_ptr(),
            null(), // Using HTTP/1.1 by default
            null(),
            null(),
            WINHTTP_FLAG_BYPASS_PROXY_CACHE | secure_flag,
        );
        if h_request.is_null() {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpOpenRequest failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let b_request_sent =
            (get_winhttp().win_http_send_request)(h_request, null(), 0, null(), 0, 0, 0);
        if b_request_sent == 0 {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_request);
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpSendRequest failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let b_response_received = (get_winhttp().win_http_receive_response)(h_request, null_mut());
        if b_response_received == 0 {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_request);
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpReceiveResponse failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let response = read_response(h_request)?;

        (get_winhttp().win_http_close_handle)(h_request);
        (get_winhttp().win_http_close_handle)(h_connect);
        (get_winhttp().win_http_close_handle)(h_session);

        Ok(response)
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
fn http_post(url: &str, path: &str, data: &str) -> Result<Response, String> {
    unsafe {
        let h_session = (get_winhttp().win_http_open)(
            to_pcwstr("RustWinHttp").as_ptr(),
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            null(),
            null(),
            0,
        );
        if h_session.is_null() {
            let error = GetLastError();
            return Err(format!(
                "WinHttpOpen failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let parsed_url = parse_url(url);

        let secure_flag = if parsed_url.scheme == 0x02 {
            WINHTTP_FLAG_SECURE
        } else {
            0
        };

        let h_connect = (get_winhttp().win_http_connect)(
            h_session,
            to_pcwstr(parsed_url.hostname.as_str()).as_ptr(),
            parsed_url.port,
            0,
        );

        if h_connect.is_null() {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpConnect failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let h_request = (get_winhttp().win_http_open_request)(
            h_connect,
            to_pcwstr("POST").as_ptr(),
            to_pcwstr(path).as_ptr(),
            null(),
            null(),
            null(),
            WINHTTP_FLAG_BYPASS_PROXY_CACHE | secure_flag,
        );
        if h_request.is_null() {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpOpenRequest failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let b_request_sent = (get_winhttp().win_http_send_request)(
            h_request,
            null(),
            0,
            data.as_ptr() as *const _,
            data.len() as u32,
            data.len() as u32,
            0,
        );
        if b_request_sent == 0 {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_request);
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpSendRequest failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let b_response_received = (get_winhttp().win_http_receive_response)(h_request, null_mut());
        if b_response_received == 0 {
            let error = GetLastError();
            (get_winhttp().win_http_close_handle)(h_request);
            (get_winhttp().win_http_close_handle)(h_connect);
            (get_winhttp().win_http_close_handle)(h_session);
            return Err(format!(
                "WinHttpReceiveResponse failed with error: {}",
                WinHttpError::from_code(error as i32)
            ));
        }

        let response = read_response(h_request)?;

        (get_winhttp().win_http_close_handle)(h_request);
        (get_winhttp().win_http_close_handle)(h_connect);
        (get_winhttp().win_http_close_handle)(h_session);

        Ok(response)
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
            }
            Err(e) => libc_println!("HTTP GET request failed: {}", e),
        }

        // Test HTTPS GET request
        match http_get("https://localhost", "/") {
            Ok(response) => {
                libc_println!("HTTPS GET request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            }
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
            }
            Err(e) => libc_println!("HTTP POST request failed: {}", e),
        }

        // Test HTTPS POST request
        match http_post("https://example.com", "/", "key=value") {
            Ok(response) => {
                libc_println!("HTTPS POST request successful!");
                libc_println!("Status Code: {}", response.status_code);
                libc_println!("Response Body: {}", response.body);
            }
            Err(e) => libc_println!("HTTPS POST request failed: {}", e),
        }
    }
}

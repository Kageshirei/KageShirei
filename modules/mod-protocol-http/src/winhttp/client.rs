use alloc::{format, string::String, sync::Arc, vec::Vec};
use alloc::string::ToString;
use core::{
    ffi::c_void,
    ptr::{null, null_mut},
    sync::atomic::{AtomicPtr, Ordering},
};

use kageshirei_communication_protocol::{error::Protocol as ProtocolError, Metadata};
use kageshirei_win32::winhttp::{WinHttpAddRequestHeadersFunc, WinHttpCloseHandleFunc, WinHttpError, WinHttpOpenRequestFunc, WinHttpReceiveResponseFunc, WinHttpSendRequestFunc, HTTP_QUERY_STATUS_CODE, WINHTTP_FLAG_BYPASS_PROXY_CACHE, WINHTTP_FLAG_SECURE, WINHTTP_QUERY_FLAG_NUMBER};
use mod_agentcore::ldr::nt_get_last_error;
use mod_win32::nt_winhttp::get_winhttp;

use super::utils::{parse_url, to_pcwstr, ParseUrlResult};

/// The WinHttpClient struct is responsible for managing WinHTTP session and connection handles.
pub struct WinHttpClient {
    /// Atomic pointer to the WinHTTP session handle.
    pub session_handle:    AtomicPtr<c_void>,
    /// Atomic pointer to the WinHTTP connection handle.
    pub connection_handle: AtomicPtr<c_void>,
    /// Vector of headers to be added to the request.
    headers: Vec<String>,
    methods: ClientMethods
}

struct ClientMethods {
    win_http_open_request: WinHttpOpenRequestFunc,
    win_http_add_request_headers: WinHttpAddRequestHeadersFunc,
    win_http_send_request: WinHttpSendRequestFunc,
    win_http_close_handle: WinHttpCloseHandleFunc,
    win_http_receive_response: WinHttpReceiveResponseFunc,
}

impl WinHttpClient {
    /// Creates a new instance of WinHttpClient.
    ///
    /// Initializes the session and connection handles to null.
    pub fn new() -> Result<Self, ProtocolError> {
        let mut instance = Self {
            session_handle:    AtomicPtr::new(null_mut()),
            connection_handle: AtomicPtr::new(null_mut()),
            headers:           Vec::new(),
            methods: ClientMethods {
                win_http_open_request: null_mut() as WinHttpOpenRequestFunc,
                win_http_add_request_headers: null_mut() as WinHttpAddRequestHeadersFunc,
                win_http_send_request: null_mut() as WinHttpSendRequestFunc,
                win_http_close_handle: null_mut() as WinHttpCloseHandleFunc,
                win_http_receive_response: null_mut() as WinHttpReceiveResponseFunc,
            }
        };

        Self::init(&mut instance)?;

        Ok(instance)
    }

    fn init(instance: &mut Self) -> Result<(), ProtocolError> {
        // TODO: Complete this function with all the remaining methods to initialize

        instance.methods.win_http_open_request = get_winhttp()
            .win_http_open_request
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_open_request method".to_string(),
            ))?;
        instance.methods.win_http_add_request_headers = get_winhttp()
            .win_http_add_request_headers
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_add_request_headers method".to_string(),
            ))?;
        instance.methods.win_http_send_request = get_winhttp()
            .win_http_send_request
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_send_request method".to_string(),
            ))?;
        instance.methods.win_http_close_handle = get_winhttp()
            .win_http_close_handle
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_close_handle method".to_string(),
            ))?;
        instance.methods.win_http_receive_response = get_winhttp()
            .win_http_receive_response
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_receive_response method".to_string(),
            ))?;

        Ok(())
    }

    /// Initializes the WinHTTP session.
    ///
    /// This function sets up the WinHTTP session handle using the specified user agent string.
    /// It ensures that the session is initialized only once.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - A string specifying the user agent.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the session is successfully initialized, or an error message
    /// if the initialization fails.
    fn init_session(&self, user_agent: &str) -> Result<(), ProtocolError> {
        if self.session_handle.load(Ordering::Acquire).is_null() {
            let win_http_open = get_winhttp()
                .win_http_open
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_open method".to_string(),
                ))?;

            unsafe {
                let h_session = win_http_open(to_pcwstr(user_agent).as_ptr(), 0, null(), null(), 0);
                self.session_handle.store(h_session, Ordering::Release);
            }
        }

        Ok(())
    }

    /// Initializes the WinHTTP connection.
    ///
    /// This function sets up the WinHTTP connection handle to the specified URL and port.
    /// It ensures that the connection is initialized only once.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to connect to.
    /// * `port` - The port number to use for the connection.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the connection is successfully initialized, or an error message
    /// if the initialization fails.
    pub fn init_connection(&self, url: &str, port: u16) -> Result<(), ProtocolError> {
        // Initialize the session with the default user agent "kageshirei-agent".
        self.init_session("kageshirei-agent")?;

        if self.connection_handle.load(Ordering::Acquire).is_null() {
            let win_http_connect = get_winhttp()
                .win_http_connect
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_connect method".to_string(),
                ))?;

            unsafe {
                let h_connect = win_http_connect(
                    self.session_handle.load(Ordering::Acquire),
                    to_pcwstr(url).as_ptr(),
                    port,
                    0,
                );
                self.connection_handle.store(h_connect, Ordering::Release);
            }
        }

        Ok(())
    }

    /// Reads the response from an HTTP request.
    ///
    /// This function queries the status code of the response and reads the response body data
    /// from the specified HTTP request handle. It constructs and returns the complete response body
    /// as a `Bytes` object. If any operation fails, it returns an error with a detailed message.
    ///
    /// # Arguments
    /// * `h_request` - A pointer to the HTTP request handle.
    ///
    /// # Returns
    /// A `Result` containing the response bytes or an error message if the read operation fails.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences raw pointers and interacts with the WinHTTP
    /// API, which requires correct usage of the API and proper management of handles.
    pub unsafe fn read_response(&self, h_request: *mut c_void) -> Result<Vec<u8>, ProtocolError> {
        let mut status_code: u32 = 0;
        let mut status_code_len: u32 = size_of::<u32>() as u32;

        let win_http_query_headers = get_winhttp()
            .win_http_query_headers
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_query_headers method".to_string(),
            ))?;
        let win_http_read_data = get_winhttp()
            .win_http_read_data
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_read_data method".to_string(),
            ))?;

        // Query the status code from the response headers.
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
            return Err(ProtocolError::Generic(format!(
                "WinHttpQueryHeaders failed with error: {}",
                WinHttpError::from_code(error as i32)
            )));
        }

        let mut buffer: [u8; 4096] = [0; 4096]; // Buffer to hold the response data.
        let mut bytes_read: u32 = 0; // Number of bytes read in each iteration.
        let mut response_body = Vec::<u8>::new(); // Mutable buffer to accumulate the response body.

        loop {
            // Read data from the response into the buffer.
            let b_result = win_http_read_data(
                h_request,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
            );

            // Break the loop if no more data is available or if the read operation fails.
            if b_result == 0 || bytes_read == 0 {
                break;
            }

            // Append the read data to the response body.
            response_body.extend_from_slice(&buffer[.. bytes_read as usize]);
        }

        // Return the complete response body as a `Bytes` object.
        Ok(response_body)
    }

    /// Mark a header to be chained to the request.
    ///
    /// NOTE: This function **must** be called before sending the request.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `WinHttpClient` instance.
    pub fn add_header(&mut self, name: String, value: String) -> &mut Self {
        self.headers.push(format!("{}: {}", name, value));

        self
    }

    async fn send_request(&self, method: String, iurl: &str, body: Vec<u8>)-> Result<Vec<u8>, ProtocolError> {
        // Parse the URL to extract the scheme, hostname, port, and path.
        let parsed_url_result: ParseUrlResult = parse_url(iurl);

        // Determine if the connection should use a secure flag based on the scheme.
        let secure_flag = if parsed_url_result.scheme == 0x02 {
            WINHTTP_FLAG_SECURE
        }
        else {
            0
        };

        // Initialize the connection to the specified hostname and port.
        self.init_connection(&parsed_url_result.hostname, parsed_url_result.port)?;



        unsafe {
            // Open a WinHTTP request handle for the POST method.
            let h_request = win_http_open_request(
                self.connection_handle.load(Ordering::Acquire),
                to_pcwstr(method.as_str()).as_ptr(),
                to_pcwstr(parsed_url_result.path.as_str()).as_ptr(),
                null(),
                null(),
                null(),
                WINHTTP_FLAG_BYPASS_PROXY_CACHE | secure_flag,
            );
            if h_request.is_null() {
                let error = nt_get_last_error();
                return Err(ProtocolError::Generic(format!(
                    "WinHttpOpenRequest failed with error: {}",
                    WinHttpError::from_code(error as i32)
                )));
            }

            // Add the headers to the request handle.
            for header in self.headers.iter() {
                let header_str = to_pcwstr(header.as_str());
                win_http_add_request_headers(h_request, header_str.as_ptr(), -1, 0);
            }

            // Send the POST request with the body data.
            let b_request_sent = win_http_send_request(
                h_request,
                null(),
                0,
                body.as_ptr() as *const _,
                body.len() as u32,
                body.len() as u32,
                0,
            );
            if b_request_sent == 0 {
                let error = nt_get_last_error();
                win_http_close_handle(h_request);
                return Err(ProtocolError::Generic(format!(
                    "WinHttpSendRequest failed with error: {}",
                    WinHttpError::from_code(error as i32)
                )));
            }

            // Receive the response to the POST request.
            let b_response_received = win_http_receive_response(h_request, null_mut());
            if b_response_received == 0 {
                let error = nt_get_last_error();
                win_http_close_handle(h_request);
                return Err(ProtocolError::Generic(format!(
                    "WinHttpReceiveResponse failed with error: {}",
                    WinHttpError::from_code(error as i32)
                )));
            }

            // Read the response from the request handle.
            let response = self.read_response(h_request)?;

            // Close the request handle after reading the response.
            win_http_close_handle(h_request);

            Ok(response)
        }
    }

    /// Sends a POST request using WinHTTP.
    ///
    /// This function sends a POST request to the specified URL with the provided body data and
    /// metadata.
    ///
    /// # Arguments
    /// * `iurl` - The URL to send the POST request to.
    /// * `body` - The body data to send in the POST request.
    /// * `metadata` - The metadata associated with the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response bytes or an error message if the request fails.
    pub async fn post(&self, iurl: &str, body: Vec<u8>) -> Result<Vec<u8>, ProtocolError> {
        self.send_request("POST".to_string(), iurl, body).await
    }
}

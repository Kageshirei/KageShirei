//! The `client` module contains the `WinHttpClient` struct, which is responsible for managing
//! WinHTTP session and connection handles.

use alloc::{
    borrow::ToOwned as _,
    format,
    string::{String},
    vec::Vec,
};
use core::{
    ffi::c_void,
    mem::transmute,
    ptr::{null, null_mut},
    sync::atomic::{AtomicPtr, Ordering},
};

use kageshirei_communication_protocol::error::Protocol as ProtocolError;
use kageshirei_win32::winhttp::{
    WinHttpAddRequestHeadersFunc,
    WinHttpCloseHandleFunc,
    WinHttpConnectFunc,
    WinHttpError,
    WinHttpOpenFunc,
    WinHttpOpenRequestFunc,
    WinHttpQueryHeadersFunc,
    WinHttpReadDataFunc,
    WinHttpReceiveResponseFunc,
    WinHttpSendRequestFunc,
    HTTP_QUERY_STATUS_CODE,
    WINHTTP_FLAG_BYPASS_PROXY_CACHE,
    WINHTTP_FLAG_SECURE,
    WINHTTP_QUERY_FLAG_NUMBER,
};
use mod_agentcore::ldr::nt_get_last_error;
use mod_win32::nt_winhttp::get_winhttp;

use super::utils::{parse_url, to_pcwstr};

/// The WinHttpClient struct is responsible for managing WinHTTP session and connection handles.
#[expect(clippy::module_name_repetitions, reason = "The module name is descriptive.")]
pub struct WinHttpClient {
    /// Atomic pointer to the WinHTTP session handle.
    pub session_handle:    AtomicPtr<c_void>,
    /// Atomic pointer to the WinHTTP connection handle.
    pub connection_handle: AtomicPtr<c_void>,
    /// Vector of headers to be added to the request.
    headers:               Vec<String>,
    /// Methods for interacting with the WinHTTP API.
    methods:               ClientMethods,
}

/// The ClientMethods struct contains function pointers for interacting with the WinHTTP API.
struct ClientMethods {
    /// The WinHttpOpenRequest function pointer.
    pub win_http_open_request:        WinHttpOpenRequestFunc,
    /// The WinHttpAddRequestHeaders function pointer.
    pub win_http_add_request_headers: WinHttpAddRequestHeadersFunc,
    /// The WinHttpSendRequest function pointer.
    pub win_http_send_request:        WinHttpSendRequestFunc,
    /// The WinHttpCloseHandle function pointer.
    pub win_http_close_handle:        WinHttpCloseHandleFunc,
    /// The WinHttpReceiveResponse function pointer.
    pub win_http_receive_response:    WinHttpReceiveResponseFunc,
    /// The WinHttpOpen function pointer.
    pub win_http_open:                WinHttpOpenFunc,
    /// The WinHttpConnect function pointer.
    pub win_http_connect:             WinHttpConnectFunc,
    /// The WinHttpQueryHeaders function pointer.
    pub win_http_query_headers:       WinHttpQueryHeadersFunc,
    /// The WinHttpReadData function pointer.
    pub win_http_read_data:           WinHttpReadDataFunc,
}

impl WinHttpClient {
    /// Creates a new instance of WinHttpClient.
    ///
    /// Initializes the session and connection handles to null.
    pub fn new() -> Result<Self, ProtocolError> {
        // Safety: The following block initializes the methods with null pointers. It's safe as immediately
        //         after this block, the `init` function is called to set the correct function pointers.
        unsafe {
            let mut instance = Self {
                session_handle:    AtomicPtr::new(null_mut()),
                connection_handle: AtomicPtr::new(null_mut()),
                headers:           Vec::new(),
                methods:           ClientMethods {
                    win_http_open_request:        transmute::<*mut c_void, WinHttpOpenRequestFunc>(null_mut()),
                    win_http_add_request_headers: transmute::<*mut c_void, WinHttpAddRequestHeadersFunc>(null_mut()),
                    win_http_send_request:        transmute::<*mut c_void, WinHttpSendRequestFunc>(null_mut()),
                    win_http_close_handle:        transmute::<*mut c_void, WinHttpCloseHandleFunc>(null_mut()),
                    win_http_receive_response:    transmute::<*mut c_void, WinHttpReceiveResponseFunc>(null_mut()),
                    win_http_open:                transmute::<*mut c_void, WinHttpOpenFunc>(null_mut()),
                    win_http_connect:             transmute::<*mut c_void, WinHttpConnectFunc>(null_mut()),
                    win_http_query_headers:       transmute::<*mut c_void, WinHttpQueryHeadersFunc>(null_mut()),
                    win_http_read_data:           transmute::<*mut c_void, WinHttpReadDataFunc>(null_mut()),
                },
            };

            Self::init(&mut instance)?;

            Ok(instance)
        }
    }

    /// Initializes the WinHttpClient instance methods.
    fn init(instance: &mut Self) -> Result<(), ProtocolError> {
        // TODO: Complete this function with all the remaining methods to initialize

        instance.methods.win_http_open = get_winhttp()
            .win_http_open
            .ok_or(ProtocolError::InitializationError(
                "Cannot find win_http_open method".to_owned(),
            ))?;

        instance.methods.win_http_connect =
            get_winhttp()
                .win_http_connect
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_connect method".to_owned(),
                ))?;

        instance.methods.win_http_query_headers =
            get_winhttp()
                .win_http_query_headers
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_query_headers method".to_owned(),
                ))?;
        instance.methods.win_http_read_data =
            get_winhttp()
                .win_http_read_data
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_read_data method".to_owned(),
                ))?;

        instance.methods.win_http_open_request =
            get_winhttp()
                .win_http_open_request
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_open_request method".to_owned(),
                ))?;
        instance.methods.win_http_add_request_headers =
            get_winhttp()
                .win_http_add_request_headers
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_add_request_headers method".to_owned(),
                ))?;
        instance.methods.win_http_send_request =
            get_winhttp()
                .win_http_send_request
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_send_request method".to_owned(),
                ))?;
        instance.methods.win_http_close_handle =
            get_winhttp()
                .win_http_close_handle
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_close_handle method".to_owned(),
                ))?;
        instance.methods.win_http_receive_response =
            get_winhttp()
                .win_http_receive_response
                .ok_or(ProtocolError::InitializationError(
                    "Cannot find win_http_receive_response method".to_owned(),
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
            // Safety: The following block interacts with the WinHTTP API and dereferences raw pointers.
            unsafe {
                let h_session = (self.methods.win_http_open)(to_pcwstr(user_agent).as_ptr(), 0, null(), null(), 0);
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
    /// A `Result` containing `()` if the connection is successfully initialized, or an error
    /// message if the initialization fails.
    pub fn init_connection(&self, url: &str, port: u16) -> Result<(), ProtocolError> {
        // Initialize the session with the default user agent "kageshirei-agent".
        self.init_session("kageshirei-agent")?;

        if self.connection_handle.load(Ordering::Acquire).is_null() {
            // Safety: The following block interacts with the WinHTTP API and dereferences raw pointers.
            unsafe {
                let h_connect = (self.methods.win_http_connect)(
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

        // Query the status code from the response headers.
        let b_status_code = (self.methods.win_http_query_headers)(
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
            let b_result = (self.methods.win_http_read_data)(
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

            let bytes_chunk = buffer.get(0 .. bytes_read as usize);
            if let Some(chunk) = bytes_chunk {
                response_body.extend_from_slice(chunk);
            }
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

    /// Sends a request using WinHTTP.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method to use for the request.
    /// * `iurl` - The URL to send the request to.
    /// * `body` - The body data to send in the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response bytes or an error message if the request fails.
    async fn send_request(&mut self, method: String, iurl: &str, body: Vec<u8>) -> Result<Vec<u8>, ProtocolError> {
        // Parse the URL to extract the scheme, hostname, port, and path.
        let parsed_url_result = parse_url(iurl);

        // Determine if the connection should use a secure flag based on the scheme.
        let secure_flag = if parsed_url_result.scheme == 0x02 {
            WINHTTP_FLAG_SECURE
        }
        else {
            0
        };

        // Initialize the connection to the specified hostname and port.
        self.init_connection(&parsed_url_result.hostname, parsed_url_result.port)?;

        // Safety: The following block interacts with the WinHTTP API and dereferences raw pointers.
        unsafe {
            // Open a WinHTTP request handle for the POST method.
            let h_request = (self.methods.win_http_open_request)(
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
                (self.methods.win_http_add_request_headers)(h_request, header_str.as_ptr(), -1, 0);
            }
            self.headers.clear();

            // Send the POST request with the body data.
            let b_request_sent = (self.methods.win_http_send_request)(
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
                (self.methods.win_http_close_handle)(h_request);
                return Err(ProtocolError::Generic(format!(
                    "WinHttpSendRequest failed with error: {}",
                    WinHttpError::from_code(error as i32)
                )));
            }

            // Receive the response to the POST request.
            let b_response_received = (self.methods.win_http_receive_response)(h_request, null_mut());
            if b_response_received == 0 {
                let error = nt_get_last_error();
                (self.methods.win_http_close_handle)(h_request);
                return Err(ProtocolError::Generic(format!(
                    "WinHttpReceiveResponse failed with error: {}",
                    WinHttpError::from_code(error as i32)
                )));
            }

            // Read the response from the request handle.
            let response = self.read_response(h_request)?;

            // Close the request handle after reading the response.
            (self.methods.win_http_close_handle)(h_request);

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
    ///
    /// # Returns
    ///
    /// A `Result` containing the response bytes or an error message if the request fails.
    pub async fn post(&mut self, iurl: &str, body: Vec<u8>) -> Result<Vec<u8>, ProtocolError> {
        self.send_request("POST".to_owned(), iurl, body).await
    }

    /// Sends a GET request using WinHTTP.
    ///
    /// This function sends a GET request to the specified URL with the provided body data and
    /// metadata.
    ///
    /// # Arguments
    /// * `iurl` - The URL to send the POST request to.
    /// * `body` - The body data to send in the POST request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response bytes or an error message if the request fails.
    pub async fn get(&mut self, iurl: &str) -> Result<Vec<u8>, ProtocolError> {
        self.send_request("GET".to_owned(), iurl, Vec::new()).await
    }
}

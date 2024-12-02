use core::{ffi::c_void, fmt};

/// WinHTTP access types and flags

/// Indicates that no proxy should be used.
pub const WINHTTP_ACCESS_TYPE_NO_PROXY: u32 = 1;

/// Uses the default proxy settings.
pub const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: u32 = 0;

/// Bypass the proxy for this request. This flag is used with the `dwFlags` parameter in
/// `WinHttpOpenRequest` and `WinHttpSendRequest`.
pub const WINHTTP_FLAG_BYPASS_PROXY_CACHE: u32 = 0x00000100;

/// Indicates that the request should use a secure HTTPS connection.
pub const WINHTTP_FLAG_SECURE: u32 = 0x00800000;

/// Indicates that the query should return numeric data. This flag is used with the
/// `dwInfoLevel` parameter in `WinHttpQueryHeaders`.
pub const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x20000000;

/// HTTP query constant for the status code. Used with the `dwInfoLevel` parameter in
/// `WinHttpQueryHeaders` to retrieve the status code of the HTTP response.
pub const HTTP_QUERY_STATUS_CODE: u32 = 19;

/// Type definition for the WinHttpOpen function.
///
/// Initializes an HTTP session for subsequent use by WinHTTP functions. This function returns
/// an HINTERNET handle required by many other WinHTTP functions.
///
/// # Parameters
/// - `pwszUserAgent`: A string that specifies the name of the application or entity using WinHTTP.
/// - `dwAccessType`: An unsigned integer that specifies the type of access to the Internet. Can be one of the following
///   values: `WINHTTP_ACCESS_TYPE_DEFAULT_PROXY` or `WINHTTP_ACCESS_TYPE_NO_PROXY`.
/// - `pwszProxyName`: A string that specifies the proxy server to use (ignored if `dwAccessType` is not
///   `WINHTTP_ACCESS_TYPE_DEFAULT_PROXY`).
/// - `pwszProxyBypass`: A string that specifies an optional list of host names or IP addresses, or both, that should
///   not be routed through the proxy when `dwAccessType` is `WINHTTP_ACCESS_TYPE_DEFAULT_PROXY`.
/// - `dwFlags`: An unsigned integer that can be set to 0 (reserved for future use).
///
/// # Returns
/// - A handle to the HTTP session, or `NULL` if the function fails.
pub type WinHttpOpenFunc = unsafe extern "system" fn(
    pwszUserAgent: *const u16,
    dwAccessType: u32,
    pwszProxyName: *const u16,
    pwszProxyBypass: *const u16,
    dwFlags: u32,
) -> *mut c_void;

/// Type definition for the WinHttpConnect function.
///
/// Creates a connection to an HTTP server for a specified session.
///
/// # Parameters
/// - `hSession`: A handle to an HTTP session returned by `WinHttpOpen`.
/// - `pswzServerName`: A string that specifies the host name of an HTTP server.
/// - `nServerPort`: An unsigned short that specifies the TCP/IP port number to connect to on the server.
/// - `dwReserved`: An unsigned integer that is reserved and must be 0.
///
/// # Returns
/// - A handle to the connection if successful, or `NULL` if the function fails.
pub type WinHttpConnectFunc = unsafe extern "system" fn(
    hSession: *mut c_void,
    pswzServerName: *const u16,
    nServerPort: u16,
    dwReserved: u32,
) -> *mut c_void;

/// Type definition for the WinHttpOpenRequest function.
///
/// Creates an HTTP request handle.
///
/// # Parameters
/// - `hConnect`: A handle to the HTTP connection returned by `WinHttpConnect`.
/// - `pwszVerb`: A string that specifies an HTTP verb, such as "GET" or "POST".
/// - `pwszObjectName`: A string that specifies the target object of the request.
/// - `pwszVersion`: A string that specifies the HTTP version (can be `NULL` to use HTTP/1.1).
/// - `pwszReferrer`: A string that specifies the referrer URL (can be `NULL`).
/// - `ppwszAcceptTypes`: A pointer to a null-terminated array of strings that specify media types accepted by the
///   client (can be `NULL`).
/// - `dwFlags`: An unsigned integer that specifies the flags for the request, such as `WINHTTP_FLAG_BYPASS_PROXY_CACHE`
///   and `WINHTTP_FLAG_SECURE`.
///
/// # Returns
/// - A handle to the HTTP request if successful, or `NULL` if the function fails.
pub type WinHttpOpenRequestFunc = unsafe extern "system" fn(
    hConnect: *mut c_void,
    pwszVerb: *const u16,
    pwszObjectName: *const u16,
    pwszVersion: *const u16,
    pwszReferrer: *const u16,
    ppwszAcceptTypes: *const *const u16,
    dwFlags: u32,
) -> *mut c_void;

/// Type definition for the WinHttpSetOption function.
///
/// Sets an option on an HINTERNET handle.
///
/// # Parameters
/// - `hInternet`: A handle on which to set the option.
/// - `dwOption`: An unsigned integer that specifies the Internet option to set.
/// - `lpBuffer`: A pointer to a buffer that contains the option setting.
/// - `dwBufferLength`: An unsigned integer that specifies the length of the buffer.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpSetOptionFunc = unsafe extern "system" fn(
    hInternet: *mut c_void,
    dwOption: u32,
    lpBuffer: *const c_void,
    dwBufferLength: u32,
) -> i32;

/// Type definition for the WinHttpCloseHandle function.
///
/// Closes an open HINTERNET handle.
///
/// # Parameters
/// - `hInternet`: A handle to be closed.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpCloseHandleFunc = unsafe extern "system" fn(hInternet: *mut c_void) -> i32;

/// Type definition for the WinHttpSendRequest function.
///
/// Sends the specified request to the HTTP server.
///
/// # Parameters
/// - `hRequest`: A handle to an HTTP request returned by `WinHttpOpenRequest`.
/// - `pwszHeaders`: A string that specifies additional headers to append to the request (can be `NULL`).
/// - `dwHeadersLength`: An unsigned integer that specifies the length of the additional headers (use `-1` to specify
///   that the headers are null-terminated).
/// - `lpOptional`: A pointer to an optional data buffer to send immediately after the request headers (can be `NULL`).
/// - `dwOptionalLength`: An unsigned integer that specifies the length of the optional data.
/// - `dwTotalLength`: An unsigned integer that specifies the total length of the request.
/// - `dwContext`: An unsigned integer that specifies an application-defined value that is passed to the callback
///   function.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpSendRequestFunc = unsafe extern "system" fn(
    hRequest: *mut c_void,
    pwszHeaders: *const u16,
    dwHeadersLength: u32,
    lpOptional: *const c_void,
    dwOptionalLength: u32,
    dwTotalLength: u32,
    dwContext: usize,
) -> i32;

/// Type definition for the WinHttpAddRequestHeaders function.
///
/// Adds one or more HTTP request headers to an HTTP request handle.
///
/// # Parameters
/// - `hRequest`: A handle to an HTTP request returned by `WinHttpOpenRequest`.
/// - `pwszHeaders`: A string that specifies the headers to append.
/// - `dwHeadersLength`: An integer that specifies the length of the headers (use `-1` to specify that the headers are
///   null-terminated).
/// - `dwModifiers`: An unsigned integer that specifies modifiers controlling the action.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpAddRequestHeadersFunc = unsafe extern "system" fn(
    hRequest: *mut c_void,
    pwszHeaders: *const u16,
    dwHeadersLength: i32,
    dwModifiers: u32,
) -> i32;

/// Type definition for the WinHttpReceiveResponse function.
///
/// Waits to receive the response to an HTTP request initiated by `WinHttpSendRequest`.
///
/// # Parameters
/// - `hRequest`: A handle to an HTTP request returned by `WinHttpOpenRequest`.
/// - `lpReserved`: A reserved parameter that must be `NULL`.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpReceiveResponseFunc = unsafe extern "system" fn(hRequest: *mut c_void, lpReserved: *mut c_void) -> i32;

/// Type definition for the WinHttpReadData function.
///
/// Reads data from a handle opened by `WinHttpOpenRequest`.
///
/// # Parameters
/// - `hRequest`: A handle to an HTTP request returned by `WinHttpOpenRequest`.
/// - `lpBuffer`: A pointer to a buffer that receives the data.
/// - `dwNumberOfBytesToRead`: An unsigned integer that specifies the number of bytes to read.
/// - `lpdwNumberOfBytesRead`: A pointer to an unsigned integer that receives the number of bytes read.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpReadDataFunc = unsafe extern "system" fn(
    hRequest: *mut c_void,
    lpBuffer: *mut c_void,
    dwNumberOfBytesToRead: u32,
    lpdwNumberOfBytesRead: *mut u32,
) -> i32;

/// Type definition for the WinHttpQueryHeaders function.
///
/// Queries information about the headers associated with an HTTP request.
///
/// # Parameters
/// - `hRequest`: A handle to an HTTP request returned by `WinHttpOpenRequest`.
/// - `dwInfoLevel`: An unsigned integer that specifies the type of information to query (e.g.,
///   `HTTP_QUERY_STATUS_CODE`).
/// - `pwszName`: A string that specifies the header name to query (can be `NULL`).
/// - `lpBuffer`: A pointer to a buffer that receives the header information.
/// - `lpdwBufferLength`: A pointer to an unsigned integer that specifies the length of the buffer.
/// - `lpdwIndex`: A pointer to an unsigned integer that specifies the header index (use `NULL` for the first
///   occurrence).
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpQueryHeadersFunc = unsafe extern "system" fn(
    hRequest: *mut c_void,
    dwInfoLevel: u32,
    pwszName: *const u16,
    lpBuffer: *mut c_void,
    lpdwBufferLength: *mut u32,
    lpdwIndex: *mut u32,
) -> i32;

/// Type definition for the WinHttpGetIEProxyConfigForCurrentUser function.
///
/// Retrieves the Internet Explorer proxy configuration for the current user.
///
/// # Parameters
/// - `pProxyConfig`: A pointer to a structure that receives the proxy configuration.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpGetIEProxyConfigForCurrentUserFunc = unsafe extern "system" fn(pProxyConfig: *mut c_void) -> i32;

/// Type definition for the WinHttpGetProxyForUrl function.
///
/// Retrieves the proxy configuration for a specified URL using the WPAD protocol or the URL provided.
///
/// # Parameters
/// - `hSession`: A handle to an HTTP session returned by `WinHttpOpen`.
/// - `lpcwszUrl`: A string that specifies the URL.
/// - `pAutoProxyOptions`: A pointer to a structure that specifies the auto-proxy options.
/// - `pProxyInfo`: A pointer to a structure that receives the proxy information.
///
/// # Returns
/// - `TRUE` if successful, or `FALSE` if the function fails.
pub type WinHttpGetProxyForUrlFunc = unsafe extern "system" fn(
    hSession: *mut c_void,
    lpcwszUrl: *const u16,
    pAutoProxyOptions: *mut c_void,
    pProxyInfo: *mut c_void,
) -> i32;

/// Structure to hold function pointers for WinHTTP API functions.
///
/// This structure contains function pointers to the various WinHTTP functions. It allows dynamic
/// loading of the functions from the WinHTTP library at runtime.
pub struct WinHttp {
    pub win_http_open: WinHttpOpenFunc,
    pub win_http_connect: WinHttpConnectFunc,
    pub win_http_open_request: WinHttpOpenRequestFunc,
    pub win_http_set_option: WinHttpSetOptionFunc,
    pub win_http_close_handle: WinHttpCloseHandleFunc,
    pub win_http_send_request: WinHttpSendRequestFunc,
    pub win_http_add_request_headers: WinHttpAddRequestHeadersFunc,
    pub win_http_receive_response: WinHttpReceiveResponseFunc,
    pub win_http_read_data: WinHttpReadDataFunc,
    pub win_http_query_headers: WinHttpQueryHeadersFunc,
    pub win_http_get_ie_proxy_config_for_current_user: WinHttpGetIEProxyConfigForCurrentUserFunc,
    pub win_http_get_proxy_for_url: WinHttpGetProxyForUrlFunc,
}

impl WinHttp {
    /// Creates a new instance of `WinHttp` with all functions initialized to null.
    ///
    /// This function initializes a new `WinHttp` instance with all function pointers set to null.
    /// The function pointers must be properly initialized before use.
    ///
    /// # Returns
    /// A new `WinHttp` instance with uninitialized function pointers.
    pub fn new() -> Self {
        WinHttp {
            win_http_open: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_connect: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_open_request: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_set_option: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_close_handle: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_send_request: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_add_request_headers: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_receive_response: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_read_data: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_query_headers: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            win_http_get_ie_proxy_config_for_current_user: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            win_http_get_proxy_for_url: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
        }
    }
}

/// Represents the error codes returned by WinHTTP functions.
#[derive(Copy, Clone, Debug)]
pub enum WinHttpError {
    /// The requested operation requires a connection to the server.
    ErrorWinhttpCannotConnect        = 12029,
    /// The connection with the server has been reset or terminated, or an incompatible SSL protocol was encountered.
    ErrorWinhttpConnectionError      = 12030,
    /// The data being supplied is of wrong type.
    ErrorWinhttpIncorrectHandleState = 12019,
    /// The request handle is invalid.
    ErrorWinhttpIncorrectHandleType  = 12018,
    /// The application made a request for a resource that the server could not understand.
    ErrorWinhttpInternalError        = 12004,
    /// An invalid option value was specified.
    ErrorWinhttpInvalidOption        = 12009,
    /// The specified option is not supported.
    ErrorWinhttpOptionNotSettable    = 12011,
    /// The application requested an operation that is not allowed when the current operation is asynchronous.
    ErrorWinhttpShutdown             = 12012,
    /// The operation timed out.
    ErrorWinhttpTimeout              = 12002,
    /// The URL is invalid.
    ErrorWinhttpUnrecognizedScheme   = 12006,
    /// The connection was aborted.
    ErrorWinhttpOperationCancelled   = 12017,
    /// The SSL certificate is invalid.
    ErrorWinhttpSecureFailure        = 12175,
    /// The SSL certificate is invalid or could not be validated.
    ErrorWinhttpSecureInvalidCert    = 12169,
    /// An unknown error occurred.
    ErrorWinhttpUnknownError         = -1,
}

impl WinHttpError {
    /// Converts an error code to a `WinHttpError`.
    ///
    /// # Arguments
    /// * `code` - The error code returned by a WinHTTP function.
    ///
    /// # Returns
    /// * A `WinHttpError` corresponding to the error code.
    pub fn from_code(code: i32) -> Self {
        match code {
            12029 => WinHttpError::ErrorWinhttpCannotConnect,
            12030 => WinHttpError::ErrorWinhttpConnectionError,
            12019 => WinHttpError::ErrorWinhttpIncorrectHandleState,
            12018 => WinHttpError::ErrorWinhttpIncorrectHandleType,
            12004 => WinHttpError::ErrorWinhttpInternalError,
            12009 => WinHttpError::ErrorWinhttpInvalidOption,
            12011 => WinHttpError::ErrorWinhttpOptionNotSettable,
            12012 => WinHttpError::ErrorWinhttpShutdown,
            12002 => WinHttpError::ErrorWinhttpTimeout,
            12006 => WinHttpError::ErrorWinhttpUnrecognizedScheme,
            12017 => WinHttpError::ErrorWinhttpOperationCancelled,
            12175 => WinHttpError::ErrorWinhttpSecureFailure,
            12169 => WinHttpError::ErrorWinhttpSecureInvalidCert,
            _ => WinHttpError::ErrorWinhttpUnknownError,
        }
    }

    /// Gets the integer value of the `WinHttpError`.
    ///
    /// # Returns
    /// * The integer value of the error code.
    pub fn code(&self) -> i32 { *self as i32 }
}

impl fmt::Display for WinHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WinHttpError::ErrorWinhttpCannotConnect => write!(f, "Cannot connect"),
            WinHttpError::ErrorWinhttpConnectionError => write!(f, "Connection error"),
            WinHttpError::ErrorWinhttpIncorrectHandleState => {
                write!(f, "Incorrect handle state")
            },
            WinHttpError::ErrorWinhttpIncorrectHandleType => write!(f, "Incorrect handle type"),
            WinHttpError::ErrorWinhttpInternalError => write!(f, "Internal error"),
            WinHttpError::ErrorWinhttpInvalidOption => write!(f, "Invalid option"),
            WinHttpError::ErrorWinhttpOptionNotSettable => write!(f, "Option not settable"),
            WinHttpError::ErrorWinhttpShutdown => write!(f, "Shutdown error"),
            WinHttpError::ErrorWinhttpTimeout => write!(f, "Operation timed out"),
            WinHttpError::ErrorWinhttpUnrecognizedScheme => write!(f, "Unrecognized scheme"),
            WinHttpError::ErrorWinhttpOperationCancelled => write!(f, "Operation cancelled"),
            WinHttpError::ErrorWinhttpSecureFailure => write!(f, "Secure failure"),
            WinHttpError::ErrorWinhttpSecureInvalidCert => write!(f, "Invalid SSL certificate"),
            WinHttpError::ErrorWinhttpUnknownError => write!(f, "Unknown error"),
        }
    }
}

impl core::error::Error for WinHttpError {}

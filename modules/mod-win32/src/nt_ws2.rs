use alloc::ffi::CString;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use mod_agentcore::instance;
use mod_agentcore::ldr::ldr_function_addr;
use rs2_win32::ntdef::{GetLastError, UnicodeString};

use core::ffi::c_void;
use core::mem::{transmute, zeroed};
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

use rs2_win32::ws2_32::{AddrInfo, SockAddr, SockAddrIn, Winsock, WsaData};

// Global variable to store Winsock functions
static mut WINSOCK_FUNCS: Option<Winsock> = None;
// Atomic variable to track if Winsock functions have been initialized
static INIT_WINSOCKS: AtomicBool = AtomicBool::new(false);

/// Initializes Winsock functions.
///
/// This function dynamically loads the Winsock functions from the "ws2_32.dll" library.
/// It ensures that the functions are loaded only once using atomic operations.
/// The function does not return any values.
pub fn init_winsock_funcs() {
    unsafe {
        if !INIT_WINSOCKS.load(Ordering::Acquire) {
            // Constants representing hash values of Winsock function names
            pub const WSA_STARTUP_DBJ2: usize = 0x142e89c3;
            pub const WSA_CLEANUP_DBJ2: usize = 0x32206eb8;
            pub const SOCKET_DBJ2: usize = 0xcf36c66e;
            pub const CONNECT_DBJ2: usize = 0xe73478ef;
            pub const SEND_DBJ2: usize = 0x7c8bc2cf;
            pub const RECV_DBJ2: usize = 0x7c8b3515;
            pub const CLOSESOCKET_DBJ2: usize = 0x185953a4;
            pub const INET_ADDR_DBJ2: usize = 0xafe73c2f;
            pub const HTONS_DBJ2: usize = 0xd454eb1;
            pub const GETADDRINFO_DBJ2: usize = 0x4b91706c;
            pub const FREEADDRINFO_DBJ2: usize = 0x307204e;

            // DLL name for Winsock
            let dll_name = "ws2_32.dll";
            let mut ws2_win32_dll_unicode = UnicodeString::new();
            let utf16_string: Vec<u16> = dll_name.encode_utf16().chain(Some(0)).collect();
            ws2_win32_dll_unicode.init(utf16_string.as_ptr());

            let mut ws2_win32_handle: *mut c_void = null_mut();

            // Load the DLL using the Windows NT Loader
            (instance().ntdll.ldr_load_dll)(
                null_mut(),
                null_mut(),
                ws2_win32_dll_unicode,
                &mut ws2_win32_handle as *mut _ as *mut c_void,
            );

            // If the handle is null, the DLL failed to load
            if ws2_win32_handle.is_null() {
                return;
            }

            let ws2_32_module = ws2_win32_handle as *mut u8;

            // Resolve function addresses using hashed names
            let wsa_startup_addr = ldr_function_addr(ws2_32_module, WSA_STARTUP_DBJ2);
            let wsa_cleanup_addr = ldr_function_addr(ws2_32_module, WSA_CLEANUP_DBJ2);
            let socket_addr = ldr_function_addr(ws2_32_module, SOCKET_DBJ2);
            let connect_addr = ldr_function_addr(ws2_32_module, CONNECT_DBJ2);
            let send_addr = ldr_function_addr(ws2_32_module, SEND_DBJ2);
            let recv_addr = ldr_function_addr(ws2_32_module, RECV_DBJ2);
            let closesocket_addr = ldr_function_addr(ws2_32_module, CLOSESOCKET_DBJ2);
            let inet_addr_addr = ldr_function_addr(ws2_32_module, INET_ADDR_DBJ2);
            let htons_addr = ldr_function_addr(ws2_32_module, HTONS_DBJ2);
            let getaddrinfo_addr = ldr_function_addr(ws2_32_module, GETADDRINFO_DBJ2);
            let freeaddrinfo_addr = ldr_function_addr(ws2_32_module, FREEADDRINFO_DBJ2);

            // Initialize Winsock functions
            let mut winsock_functions = Winsock::new();
            winsock_functions.wsa_startup = transmute(wsa_startup_addr);
            winsock_functions.wsa_cleanup = transmute(wsa_cleanup_addr);
            winsock_functions.socket = transmute(socket_addr);
            winsock_functions.connect = transmute(connect_addr);
            winsock_functions.send = transmute(send_addr);
            winsock_functions.recv = transmute(recv_addr);
            winsock_functions.closesocket = transmute(closesocket_addr);
            winsock_functions.inet_addr = transmute(inet_addr_addr);
            winsock_functions.htons = transmute(htons_addr);
            winsock_functions.getaddrinfo = transmute(getaddrinfo_addr);
            winsock_functions.freeaddrinfo = transmute(freeaddrinfo_addr);

            // Store the functions in the global variable
            WINSOCK_FUNCS = Some(winsock_functions);

            // Mark Winsock functions as initialized
            INIT_WINSOCKS.store(true, Ordering::Release);
        }
    }
}

/// Gets the Winsock functions.
///
/// This function ensures the Winsock functions are initialized and returns a reference to them.
/// If the functions are not already initialized, it will initialize them first.
///
/// # Returns
/// * `&'static Winsock` - A reference to the initialized Winsock functions.
pub fn get_winsock() -> &'static Winsock {
    init_winsock_funcs();
    return unsafe { WINSOCK_FUNCS.as_ref().unwrap() };
}

/// Initializes the Winsock library.
///
/// This function initializes the Winsock library, which is required for network operations on Windows.
/// It returns a `Result` indicating whether the initialization was successful or not.
///
/// # Returns
/// * `Ok(())` if the Winsock library was successfully initialized.
/// * `Err(String)` if there was an error during initialization, with the error message.
pub fn init_winsock() -> Result<(), String> {
    unsafe {
        let mut wsa_data: WsaData = core::mem::zeroed();
        let result = (get_winsock().wsa_startup)(0x0202, &mut wsa_data); // Initialize Winsock with version 2.2
        if result != 0 {
            return Err(format!("WSAStartup failed with error code: {}", result));
        }
    }
    Ok(())
}

/// Cleans up the Winsock library.
///
/// This function cleans up the Winsock library, releasing any resources that were allocated.
/// This function does not return any values.
pub fn cleanup_winsock() {
    unsafe {
        (get_winsock().wsa_cleanup)();
    }
}

/// Creates a new socket.
///
/// This function creates a new socket for network communication using the specified parameters.
/// It returns a `Result` with the socket descriptor or an error message if the creation failed.
///
/// # Returns
/// * `Ok(u32)` with the socket descriptor if the socket was successfully created.
/// * `Err(String)` if there was an error during socket creation, with the error message.
pub fn create_socket() -> Result<u32, String> {
    unsafe {
        let sock = (get_winsock().socket)(2, 1, 6); // Create a socket with AF_INET, SOCK_STREAM, IPPROTO_TCP
        if sock == u32::MAX {
            let error_code = GetLastError();
            return Err(format!(
                "Socket creation failed with error code: {}",
                error_code
            ));
        }
        Ok(sock)
    }
}

/// Resolves a hostname to an IPv4 address.
///
/// This function resolves a given hostname to its corresponding IPv4 address using the getaddrinfo function.
/// It returns a `Result` with the IPv4 address or an error message if the resolution failed.
///
/// # Arguments
/// * `hostname` - The hostname to be resolved.
///
/// # Returns
/// * `Ok(u32)` with the IPv4 address if the resolution was successful.
/// * `Err(String)` if there was an error during resolution, with the error message.
pub fn resolve_hostname(hostname: &str) -> Result<u32, String> {
    unsafe {
        let hostname_cstr = CString::new(hostname).unwrap();
        let mut hints: AddrInfo = zeroed();
        hints.ai_family = 2; // AF_INET
        hints.ai_socktype = 1; // SOCK_STREAM
        let mut res: *mut AddrInfo = null_mut();

        let status = (get_winsock().getaddrinfo)(hostname_cstr.as_ptr(), null(), &hints, &mut res);
        if status != 0 {
            return Err(format!("getaddrinfo failed with error code: {}", status));
        }

        if res.is_null() {
            return Err("getaddrinfo returned null".to_string());
        }

        let mut ip_addr: u32 = 0;
        let mut addr_info_ptr = res;

        while !addr_info_ptr.is_null() {
            let addr_info = &*addr_info_ptr;
            if addr_info.ai_family == 2 {
                // AF_INET
                let sockaddr_in = &*(addr_info.ai_addr as *const SockAddrIn);
                ip_addr = sockaddr_in.sin_addr.s_addr;
                break;
            }
            addr_info_ptr = addr_info.ai_next;
        }

        (get_winsock().freeaddrinfo)(res);

        if ip_addr == 0 {
            return Err("No valid IPv4 address found".to_string());
        }

        Ok(ip_addr)
    }
}

/// Connects a socket to a specified address and port.
///
/// This function connects a socket to the specified address and port.
/// It returns a `Result` indicating whether the connection was successful or not.
///
/// # Arguments
/// * `sock` - The socket descriptor.
/// * `addr` - The address to connect to.
/// * `port` - The port number to connect to.
///
/// # Returns
/// * `Ok(())` if the connection was successful.
/// * `Err(String)` if there was an error during connection, with the error message.
pub fn connect_socket(sock: u32, addr: &str, port: u16) -> Result<(), String> {
    unsafe {
        let addr = if addr == "localhost" {
            "127.0.0.1"
        } else {
            addr
        };

        let resolve_addr = resolve_hostname(addr).unwrap();
        let mut sockaddr_in: SockAddrIn = core::mem::zeroed();
        sockaddr_in.sin_family = 2; // AF_INET
        sockaddr_in.sin_port = (get_winsock().htons)(port);
        sockaddr_in.sin_addr.s_addr = resolve_addr;

        let sockaddr = &sockaddr_in as *const _ as *const SockAddr;
        let result =
            (get_winsock().connect)(sock, sockaddr, core::mem::size_of::<SockAddrIn>() as i32);
        if result != 0 {
            let error_code = GetLastError();
            return Err(format!(
                "Socket connection failed with error code: {}",
                error_code
            ));
        }
    }
    Ok(())
}

/// Sends a request through a socket.
///
/// This function sends a request through the specified socket.
/// It returns a `Result` indicating whether the send operation was successful or not.
///
/// # Arguments
/// * `sock` - The socket descriptor.
/// * `request` - The request data to be sent.
///
/// # Returns
/// * `Ok(())` if the request was successfully sent.
/// * `Err(String)` if there was an error during the send operation, with the error message.
pub fn send_request(sock: u32, request: &[u8]) -> Result<(), String> {
    unsafe {
        let result =
            (get_winsock().send)(sock, request.as_ptr() as *const i8, request.len() as i32, 0);
        if result == -1 {
            let error_code = GetLastError();
            return Err(format!(
                "Send request failed with error code: {}",
                error_code
            ));
        }
    }
    Ok(())
}

/// Receives a response from a socket.
///
/// This function receives a response from the specified socket.
/// It returns a `Result` with the response string or an error message if the receive operation failed.
///
/// # Arguments
/// * `sock` - The socket descriptor.
///
/// # Returns
/// * `Ok(String)` with the response if it was successfully received.
/// * `Err(String)` if there was an error during the receive operation, with the error message.
pub fn receive_response(sock: u32) -> Result<String, String> {
    unsafe {
        let mut buffer = [0u8; 4096];
        let mut response = String::new();
        loop {
            let bytes_received =
                (get_winsock().recv)(sock, buffer.as_mut_ptr() as *mut i8, buffer.len() as i32, 0);
            if bytes_received == -1 {
                let error_code = GetLastError();
                return Err(format!(
                    "Receive response failed with error code: {}",
                    error_code
                ));
            } else if bytes_received == 0 {
                break;
            } else {
                response.push_str(&String::from_utf8_lossy(&buffer[..bytes_received as usize]));
            }
        }
        Ok(response)
    }
}

/// Performs an HTTP GET request.
///
/// This function performs an HTTP GET request to the specified URL and path.
/// It returns a `Result` with the response string or an error message if the request failed.
///
/// # Arguments
/// * `url` - The URL to send the GET request to.
/// * `path` - The path to request.
///
/// # Returns
/// * `Ok(String)` with the response if the request was successful.
/// * `Err(String)` if there was an error during the request, with the error message.
pub fn http_get(url: &str, path: &str) -> Result<String, String> {
    init_winsock()?;
    let sock = create_socket()?;
    connect_socket(sock, url, 80)?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, url
    );
    send_request(sock, &request.as_bytes())?;

    let response = receive_response(sock)?;
    unsafe {
        (get_winsock().closesocket)(sock);
    }
    cleanup_winsock();
    Ok(response)
}

/// Performs an HTTP POST request.
///
/// This function performs an HTTP POST request to the specified URL and path with the given data.
/// It returns a `Result` with the response string or an error message if the request failed.
///
/// # Arguments
/// * `url` - The URL to send the POST request to.
/// * `path` - The path to request.
/// * `data` - The data to be sent in the POST request.
///
/// # Returns
/// * `Ok(String)` with the response if the request was successful.
/// * `Err(String)` if there was an error during the request, with the error message.
pub fn http_post(url: &str, path: &str, data: &str) -> Result<String, String> {
    init_winsock()?;
    let sock = create_socket()?;
    connect_socket(sock, url, 80)?;

    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nConnection: close\r\n\r\n{}",
        path, url, data.len(), data
    );
    send_request(sock, &request.as_bytes())?;

    let response = receive_response(sock)?;
    unsafe {
        (get_winsock().closesocket)(sock);
    }
    cleanup_winsock();
    Ok(response)
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

    #[test]
    fn test_http_get() {
        match http_get("localhost", "/") {
            Ok(response) => {
                libc_println!("GET request successful!");
                libc_println!("Response: {}", response);
            }
            Err(e) => libc_println!("GET request failed: {}", e),
        }
    }

    #[test]
    fn test_http_post() {
        match http_post("localhost", "/", "key=value") {
            Ok(response) => {
                libc_println!("POST request successful!");
                libc_println!("Response: {}", response);
            }
            Err(e) => libc_println!("POST request failed: {}", e),
        }
    }
}

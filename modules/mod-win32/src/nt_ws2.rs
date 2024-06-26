use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use core::ffi::c_void;
use core::mem::transmute;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

use mod_agentcore::instance;
use mod_agentcore::ldr::ldr_function_addr;
use rs2_win32::ntdef::UnicodeString;
use rs2_win32::ws2_32::{SockAddr, SockAddrIn, Winsock, WsaData};

extern "system" {
    fn GetLastError() -> u32;
}

// Global state for Winsock functions
static mut WINSOCK_FUNCS: Option<Winsock> = None;
static INIT_WINSOCKS: AtomicBool = AtomicBool::new(false);

fn init_winsock_funcs() {
    unsafe {
        if !INIT_WINSOCKS.load(Ordering::Acquire) {
            pub const WSA_STARTUP_DBJ2: usize = 0x142e89c3;
            pub const WSA_CLEANUP_DBJ2: usize = 0x32206eb8;
            pub const SOCKET_DBJ2: usize = 0xcf36c66e;
            pub const CONNECT_DBJ2: usize = 0xe73478ef;
            pub const SEND_DBJ2: usize = 0x7c8bc2cf;
            pub const RECV_DBJ2: usize = 0x7c8b3515;
            pub const CLOSESOCKET_DBJ2: usize = 0x185953a4;
            pub const INET_ADDR_DBJ2: usize = 0xafe73c2f;
            pub const HTONS_DBJ2: usize = 0xd454eb1;

            // Load ws2_32.dll Dll
            let dll_name = "ws2_32.dll";
            let mut ws2_win32_dll_unicode = UnicodeString::new();
            let utf16_string: Vec<u16> = dll_name.encode_utf16().chain(Some(0)).collect();
            ws2_win32_dll_unicode.init(utf16_string.as_ptr());

            let mut ws2_win32_handle: *mut c_void = null_mut();

            (instance().ntdll.ldr_load_dll)(
                null_mut(),
                null_mut(),
                ws2_win32_dll_unicode,
                &mut ws2_win32_handle as *mut _ as *mut c_void,
            );

            if ws2_win32_handle.is_null() {
                return;
            }

            let ws2_32_module = ws2_win32_handle as *mut u8;

            // Resolve function addresses
            let wsa_startup_addr = ldr_function_addr(ws2_32_module, WSA_STARTUP_DBJ2);
            let wsa_cleanup_addr = ldr_function_addr(ws2_32_module, WSA_CLEANUP_DBJ2);
            let socket_addr = ldr_function_addr(ws2_32_module, SOCKET_DBJ2);
            let connect_addr = ldr_function_addr(ws2_32_module, CONNECT_DBJ2);
            let send_addr = ldr_function_addr(ws2_32_module, SEND_DBJ2);
            let recv_addr = ldr_function_addr(ws2_32_module, RECV_DBJ2);
            let closesocket_addr = ldr_function_addr(ws2_32_module, CLOSESOCKET_DBJ2);
            let inet_addr_addr = ldr_function_addr(ws2_32_module, INET_ADDR_DBJ2);
            let htons_addr = ldr_function_addr(ws2_32_module, HTONS_DBJ2);

            let mut winsock_functions = Winsock::new();

            // Transmute function addresses to their respective types
            winsock_functions.wsa_startup = transmute(wsa_startup_addr);
            winsock_functions.wsa_cleanup = transmute(wsa_cleanup_addr);
            winsock_functions.socket = transmute(socket_addr);
            winsock_functions.connect = transmute(connect_addr);
            winsock_functions.send = transmute(send_addr);
            winsock_functions.recv = transmute(recv_addr);
            winsock_functions.closesocket = transmute(closesocket_addr);
            winsock_functions.inet_addr = transmute(inet_addr_addr);
            winsock_functions.htons = transmute(htons_addr);

            // Store function pointers in global state
            WINSOCK_FUNCS = Some(winsock_functions);

            INIT_WINSOCKS.store(true, Ordering::Release);
        }
    }
}

// Initialize Winsock
fn init_winsock() -> Result<(), String> {
    unsafe {
        init_winsock_funcs();
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();

        let mut wsa_data: WsaData = core::mem::zeroed();
        let result = (funcs.wsa_startup)(0x0202, &mut wsa_data);
        if result != 0 {
            return Err(format!("WSAStartup failed with error code: {}", result));
        }
    }
    Ok(())
}

// Cleanup Winsock
fn cleanup_winsock() {
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        (funcs.wsa_cleanup)();
    }
}

// Create a socket
fn create_socket() -> Result<u32, String> {
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        let sock = (funcs.socket)(2, 1, 6); // AF_INET, SOCK_STREAM, IPPROTO_TCP
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

// Connect to a socket
fn connect_socket(sock: u32, addr: &str, port: u16) -> Result<(), String> {
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        let addr = if addr == "localhost" {
            "127.0.0.1"
        } else {
            addr
        };
        let addr_cstr = CString::new(addr).unwrap();
        let mut sockaddr_in: SockAddrIn = core::mem::zeroed();
        sockaddr_in.sin_family = 2; // AF_INET
        sockaddr_in.sin_port = (funcs.htons)(port);
        sockaddr_in.sin_addr.s_addr = (funcs.inet_addr)(addr_cstr.as_ptr());

        let sockaddr = &sockaddr_in as *const _ as *const SockAddr;
        let result = (funcs.connect)(sock, sockaddr, core::mem::size_of::<SockAddrIn>() as i32);
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

// Send a request
fn send_request(sock: u32, request: &str) -> Result<(), String> {
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        let result = (funcs.send)(sock, request.as_ptr() as *const i8, request.len() as i32, 0);
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

// Receive a response
fn receive_response(sock: u32) -> Result<String, String> {
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        let mut buffer = [0u8; 4096];
        let mut response = String::new();
        loop {
            let bytes_received =
                (funcs.recv)(sock, buffer.as_mut_ptr() as *mut i8, buffer.len() as i32, 0);
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

// Perform HTTP GET request
pub fn http_get(url: &str, path: &str) -> Result<String, String> {
    init_winsock()?;
    let sock = create_socket()?;
    connect_socket(sock, url, 80)?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, url
    );
    send_request(sock, &request)?;

    let response = receive_response(sock)?;
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        (funcs.closesocket)(sock);
    }
    cleanup_winsock();
    Ok(response)
}

// Perform HTTP POST request
pub fn http_post(url: &str, path: &str, data: &str) -> Result<String, String> {
    init_winsock()?;
    let sock = create_socket()?;
    connect_socket(sock, url, 80)?;

    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nConnection: close\r\n\r\n{}",
        path, url, data.len(), data
    );
    send_request(sock, &request)?;

    let response = receive_response(sock)?;
    unsafe {
        let funcs = WINSOCK_FUNCS.as_ref().unwrap();
        (funcs.closesocket)(sock);
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

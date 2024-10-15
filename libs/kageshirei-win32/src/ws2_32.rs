#[expect(non_camel_case_types)]
pub type SOCKET = usize;
// Data structures for Winsock
#[repr(C)]
pub struct WsaData {
    pub w_version:        u16,
    pub w_high_version:   u16,
    pub sz_description:   [i8; 257],
    pub sz_system_status: [i8; 129],
    pub i_max_sockets:    u16,
    pub i_max_udp_dg:     u16,
    pub lp_vendor_info:   *mut i8,
}

#[repr(C)]
pub struct SockAddrIn {
    pub sin_family: u16,
    pub sin_port:   u16,
    pub sin_addr:   InAddr,
    pub sin_zero:   [i8; 8],
}

#[repr(C)]
pub struct InAddr {
    pub s_addr: u32,
}

#[repr(C)]
pub struct SockAddr {
    pub sa_family: u16,
    pub sa_data:   [i8; 14],
}

#[repr(C)]
pub struct AddrInfo {
    pub ai_flags:     i32,
    pub ai_family:    i32,
    pub ai_socktype:  i32,
    pub ai_protocol:  i32,
    pub ai_addrlen:   u32,
    pub ai_canonname: *mut i8,
    pub ai_addr:      *mut SockAddr,
    pub ai_next:      *mut AddrInfo,
}

#[expect(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FD_SET {
    pub fd_count: u32,
    pub fd_array: [SOCKET; 64],
}

#[expect(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TIMEVAL {
    pub tv_sec:  i32,
    pub tv_usec: i32,
}

// Define function types for Winsock functions
type WSAStartupFunc = unsafe extern "system" fn(wVersionRequested: u16, lpWsaData: *mut WsaData) -> i32;
type WSACleanupFunc = unsafe extern "system" fn() -> i32;
type SocketFunc = unsafe extern "system" fn(af: i32, socket_type: i32, protocol: i32) -> SOCKET;
type ConnectFunc = unsafe extern "system" fn(s: SOCKET, name: *const SockAddr, namelen: i32) -> i32;
type SendFunc = unsafe extern "system" fn(s: SOCKET, buf: *const i8, len: i32, flags: i32) -> i32;
type RecvFunc = unsafe extern "system" fn(s: SOCKET, buf: *mut i8, len: i32, flags: i32) -> i32;
type ClosesocketFunc = unsafe extern "system" fn(s: SOCKET) -> i32;
type InetAddrFunc = unsafe extern "system" fn(cp: *const i8) -> u32;
type HtonsFunc = unsafe extern "system" fn(hostshort: u16) -> u16;
type GetAddrInfoFunc = unsafe extern "system" fn(
    node: *const i8,
    service: *const i8,
    hints: *const AddrInfo,
    res: *mut *mut AddrInfo,
) -> i32;
type FreeAddrInfoFunc = unsafe extern "system" fn(res: *mut AddrInfo);
type Ioctlsocket = unsafe extern "system" fn(s: SOCKET, cmd: i32, argp: *mut u32) -> i32;

type Select = unsafe extern "system" fn(
    nfds: i32,
    readfds: *mut FD_SET,
    writefds: *mut FD_SET,
    exceptfds: *mut FD_SET,
    timeout: *mut TIMEVAL,
) -> i32;
type WSAGetLastError = unsafe extern "system" fn() -> i32;

// Structure to hold function pointers
pub struct Winsock {
    pub wsa_startup:        WSAStartupFunc,
    pub wsa_cleanup:        WSACleanupFunc,
    pub socket:             SocketFunc,
    pub connect:            ConnectFunc,
    pub send:               SendFunc,
    pub recv:               RecvFunc,
    pub closesocket:        ClosesocketFunc,
    pub inet_addr:          InetAddrFunc,
    pub htons:              HtonsFunc,
    pub getaddrinfo:        GetAddrInfoFunc,
    pub freeaddrinfo:       FreeAddrInfoFunc,
    pub ioctlsocket:        Ioctlsocket,
    pub select:             Select,
    pub wsa_get_last_error: WSAGetLastError,
}

impl Default for Winsock {
    fn default() -> Self { Self::new() }
}

impl Winsock {
    // Function to initialize the Winsock structure with null values
    pub fn new() -> Self {
        Self {
            wsa_startup:        unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            wsa_cleanup:        unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            socket:             unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            connect:            unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            send:               unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            recv:               unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            closesocket:        unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            inet_addr:          unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            htons:              unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            getaddrinfo:        unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            freeaddrinfo:       unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            ioctlsocket:        unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            select:             unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
            wsa_get_last_error: unsafe { core::mem::transmute(core::ptr::null::<core::ffi::c_void>()) },
        }
    }
}

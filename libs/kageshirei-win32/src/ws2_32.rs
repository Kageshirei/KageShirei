pub type SOCKET = usize;

/// The WSADATA structure contains information about the Windows Sockets implementation.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/ns-winsock2-wsadata
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct WsaData {
    pub w_version:        u16,
    pub w_high_version:   u16,
    pub sz_description:   [i8; 257],
    pub sz_system_status: [i8; 129],
    pub i_max_sockets:    u16,
    pub i_max_udp_dg:     u16,
    pub lp_vendor_info:   *mut i8,
}

/// The SOCKADDR_IN structure specifies a transport address and port for the AF_INET address family.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/ws2def/ns-ws2def-sockaddr_in
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct SockAddrIn {
    pub sin_family: u16,
    pub sin_port:   u16,
    pub sin_addr:   InAddr,
    pub sin_zero:   [i8; 8],
}

/// The in_addr structure represents an IPv4 Internet address.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/ns-winsock2-in_addr
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct InAddr {
    pub s_addr: u32,
}

/// The sockaddr structure varies depending on the protocol selected.
///
/// Except for the sin*_family parameter, sockaddr contents are expressed in network byte order.
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct SockAddr {
    pub sa_family: u16,
    pub sa_data:   [i8; 14],
}

/// The addrinfo structure is used by the getaddrinfo function to hold host address information.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/ws2def/ns-ws2def-addrinfoa
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
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

/// The FD_SET structure contains an array of sockets to be checked for a particular condition.
///
/// The fd_set structure is used by various Windows Sockets functions and service providers, such as
/// the select function, to place sockets into a "set" for various purposes, such as testing a given
/// socket for readability using the readfds parameter of the select function.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/ns-winsock2-fd_set
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct FD_SET {
    pub fd_count: u32,
    pub fd_array: [SOCKET; 64],
}

/// The timeval structure is used to specify a time interval.
///
/// It is associated with the Berkeley Software Distribution (BSD) Time.h header file.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/ns-winsock2-timeval
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct TIMEVAL {
    pub tv_sec:  i32,
    pub tv_usec: i32,
}

/// The WSAStartup function initiates use of the Winsock DLL by a process.
///
/// int WSAAPI WSAStartup(
///   [in]  WORD      wVersionRequested,
///   [out] LPWSADATA lpWSAData
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsastartup
pub type WSAStartupFunc = unsafe extern "system" fn(wVersionRequested: u16, lpWsaData: *mut WsaData) -> i32;

/// The WSACleanup function terminates use of the Winsock 2 DLL (Ws2_32.dll).
///
/// int WSAAPI WSACleanup();
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsacleanup
pub type WSACleanupFunc = unsafe extern "system" fn() -> i32;

/// The socket function creates a socket that is bound to a specific transport service provider.
///
/// SOCKET WSAAPI socket(
///   [in] int af,
///   [in] int type,
///   [in] int protocol
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-socket
pub type SocketFunc = unsafe extern "system" fn(af: i32, socket_type: i32, protocol: i32) -> SOCKET;

/// The connect function establishes a connection to a specified socket.
///
/// int WSAAPI connect(
///   [in] SOCKET         s,
///   [in] const sockaddr *name,
///   [in] int            namelen
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
pub type ConnectFunc = unsafe extern "system" fn(s: SOCKET, name: *const SockAddr, namelen: i32) -> i32;

/// The send function sends data on a connected socket.
///
/// int WSAAPI send(
///   [in] SOCKET     s,
///   [in] const char *buf,
///   [in] int        len,
///   [in] int        flags
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-send
pub type SendFunc = unsafe extern "system" fn(s: SOCKET, buf: *const i8, len: i32, flags: i32) -> i32;

/// The recv function receives data from a connected socket or a bound connectionless socket.
///
/// int WSAAPI recv(
///   [in]  SOCKET s,
///   [out] char   *buf,
///   [in]  int    len,
///   [in]  int    flags
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-recv
pub type RecvFunc = unsafe extern "system" fn(s: SOCKET, buf: *mut i8, len: i32, flags: i32) -> i32;

/// The closesocket function closes an existing socket.
///
/// int closesocket(
///   [in] SOCKET s
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-closesocket
pub type CloseSocketFunc = unsafe extern "system" fn(s: SOCKET) -> i32;

/// The inet_addr function converts a string containing an IPv4 dotted-decimal address into a proper
/// address for the IN_ADDR structure.
///
/// unsigned long WSAAPI inet_addr(
///   const char *cp
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-inet_addr
pub type InetAddrFunc = unsafe extern "system" fn(cp: *const i8) -> u32;

/// The htons function converts a u_short from host to TCP/IP network byte order (which is
/// big-endian).
///
/// u_short htons(
///   [in] u_short hostshort
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-htons
pub type HtonsFunc = unsafe extern "system" fn(hostshort: u16) -> u16;

/// The getaddrinfo function provides protocol-independent translation from an ANSI host name to an
/// address.
///
/// INT WSAAPI getaddrinfo(
///   [in, optional] PCSTR           pNodeName,
///   [in, optional] PCSTR           pServiceName,
///   [in, optional] const ADDRINFOA *pHints,
///   [out]          PADDRINFOA      *ppResult
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfo
pub type GetAddrInfoFunc = unsafe extern "system" fn(
    node: *const i8,
    service: *const i8,
    hints: *const AddrInfo,
    res: *mut *mut AddrInfo,
) -> i32;

/// The freeaddrinfo function frees address information that the getaddrinfo function dynamically
/// allocates in addrinfo structures.
///
/// VOID WSAAPI freeaddrinfo(
///   [in] PADDRINFOA pAddrInfo
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-freeaddrinfo
pub type FreeAddrInfoFunc = unsafe extern "system" fn(res: *mut AddrInfo);

/// The ioctlsocket function controls the I/O mode of a socket.
///
/// int WSAAPI ioctlsocket(
///   [in]      SOCKET s,
///   [in]      long   cmd,
///   [in, out] u_long *argp
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-ioctlsocket
pub type IoctlsocketFunc = unsafe extern "system" fn(s: SOCKET, cmd: i32, argp: *mut u32) -> i32;

/// The select function determines the status of one or more sockets, waiting if necessary, to
/// perform synchronous I/O.
///
/// int WSAAPI select(
///   [in]      int           nfds,
///   [in, out] fd_set        *readfds,
///   [in, out] fd_set        *writefds,
///   [in, out] fd_set        *exceptfds,
///   [in]      const timeval *timeout
/// );
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-select
pub type SelectFunc = unsafe extern "system" fn(
    nfds: i32,
    readfds: *mut FD_SET,
    writefds: *mut FD_SET,
    exceptfds: *mut FD_SET,
    timeout: *mut TIMEVAL,
) -> i32;

/// The WSAGetLastError function returns the error status for the last Windows Sockets operation
/// that failed.
///
/// int WSAAPI WSAGetLastError();
///
/// https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsagetlasterror
pub type WSAGetLastErrorFunc = unsafe extern "system" fn() -> i32;

// Structure to hold function pointers
pub struct Winsock {
    pub wsa_startup:        Option<WSAStartupFunc>,
    pub wsa_cleanup:        Option<WSACleanupFunc>,
    pub socket:             Option<SocketFunc>,
    pub connect:            Option<ConnectFunc>,
    pub send:               Option<SendFunc>,
    pub recv:               Option<RecvFunc>,
    pub closesocket:        Option<CloseSocketFunc>,
    pub inet_addr:          Option<InetAddrFunc>,
    pub htons:              Option<HtonsFunc>,
    pub getaddrinfo:        Option<GetAddrInfoFunc>,
    pub freeaddrinfo:       Option<FreeAddrInfoFunc>,
    pub ioctlsocket:        Option<IoctlsocketFunc>,
    pub select:             Option<SelectFunc>,
    pub wsa_get_last_error: Option<WSAGetLastErrorFunc>,
}

impl Default for Winsock {
    fn default() -> Self { Self::new() }
}

impl Winsock {
    // Function to initialize the Winsock structure with null values
    pub fn new() -> Self {
        Self {
            wsa_startup:        None,
            wsa_cleanup:        None,
            socket:             None,
            connect:            None,
            send:               None,
            recv:               None,
            closesocket:        None,
            inet_addr:          None,
            htons:              None,
            getaddrinfo:        None,
            freeaddrinfo:       None,
            ioctlsocket:        None,
            select:             None,
            wsa_get_last_error: None,
        }
    }
}

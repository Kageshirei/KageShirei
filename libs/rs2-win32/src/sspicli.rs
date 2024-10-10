use core::ffi::{c_ulong, c_void};

pub const SECURITY_NETWORK_DREP: c_ulong = 0x00000000;
pub const SCHANNEL_CRED_VERSION: c_ulong = 4;

pub const ISC_REQ_CONFIDENTIALITY: c_ulong = 0x00000010;
pub const ISC_REQ_ALLOCATE_MEMORY: c_ulong = 0x00000100;
pub const ISC_REQ_STREAM: c_ulong = 0x00008000;
pub const ISC_REQ_EXTENDED_ERROR: c_ulong = 0x00004000;
pub const ISC_REQ_SEQUENCE_DETECT: c_ulong = 0x00000008;
pub const ISC_REQ_REPLAY_DETECT: c_ulong = 0x00000004;
pub const ISC_RET_EXTENDED_ERROR: u32 = 0x00004000;

pub const SCH_CRED_NO_DEFAULT_CREDS: c_ulong = 0x00000010;
pub const SCH_CRED_MANUAL_CRED_VALIDATION: c_ulong = 0x00000008;
pub const SCH_CRED_AUTO_CRED_VALIDATION: c_ulong = 0x00000020;

pub const SECBUFFER_DATA: c_ulong = 1;
pub const SECBUFFER_EMPTY: c_ulong = 0;
pub const SECBUFFER_VERSION: c_ulong = 0;
pub const SECBUFFER_STREAM_HEADER: c_ulong = 7;
pub const SECBUFFER_STREAM_TRAILER: c_ulong = 6;
pub const SECBUFFER_TOKEN: c_ulong = 2;
pub const SECBUFFER_EXTRA: c_ulong = 5;

pub const SECURITY_NATIVE_DREP: u32 = 0x00000010;

pub const SEC_E_INCOMPLETE_MESSAGE: u32 = 0x80090318;
pub const SEC_E_OK: u32 = 0x00000000;
pub const SEC_I_CONTEXT_EXPIRED: u32 = 0x00090317;
pub const SEC_I_INCOMPLETE_CREDENTIALS: u32 = 0x00090320;
pub const SEC_I_CONTINUE_NEEDED: c_ulong = 0x00090312;
pub const SEC_I_COMPLETE_NEEDED: c_ulong = 0x00090313;
pub const SEC_I_COMPLETE_AND_CONTINUE: c_ulong = 0x00090314;
pub const SEC_I_RENEGOTIATE: c_ulong = 0x00090321;

#[repr(C)]
pub struct SecHandle {
    pub dw_lower: usize,
    pub dw_upper: usize,
}

pub type CredHandle = SecHandle;
pub type CtxtHandle = SecHandle;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SecBuffer {
    pub cb_buffer: u32,
    pub buffer_type: u32,
    pub pv_buffer: *mut c_void,
}

#[repr(C)]
pub struct SecBufferDesc {
    pub ul_version: u32,
    pub c_buffers: u32,
    pub p_buffers: *mut SecBuffer,
}

#[repr(C)]
pub struct TimeStamp {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

#[repr(C)]
pub struct SchannelCred {
    pub dw_version: c_ulong,
    pub c_creds: c_ulong,
    pub pa_cred: *const *const c_void,
    pub h_root_store: *const c_void,
    pub c_mappers: c_ulong,
    pub aph_mappers: *const *const c_void,
    pub c_supported_algs: c_ulong,
    pub palg_supported_algs: *const c_ulong,
    pub grbit_enabled_protocols: c_ulong,
    pub dw_minimum_cipher_strength: c_ulong,
    pub dw_maximum_cipher_strength: c_ulong,
    pub dw_session_lifespan: c_ulong,
    pub dw_flags: c_ulong,
    pub reserved: c_ulong,
}

// Costanti per gli attributi del pacchetto di sicurezza
pub const SECPKG_ATTR_STREAM_SIZES: u32 = 0;

// Struttura per SecPkgContext_StreamSizes
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SecPkgContextStreamSizes {
    pub cb_header: u32,
    pub cb_trailer: u32,
    pub cb_maximum_message: u32,
    pub c_buffers: u32,
    pub cb_block_size: u32,
}

pub type AcquireCredentialsHandleWFunc = unsafe extern "system" fn(
    pszPrincipal: *const u16, // SEC_WCHAR
    pszPackage: *const u16,   // SEC_WCHAR
    fCredentialUse: u32,
    pvLogonId: *mut c_void,
    pAuthData: *mut c_void,
    pGetKeyFn: *mut c_void,
    pvGetKeyArgument: *mut c_void,
    phCredential: *mut CredHandle,
    ptsExpiry: *mut TimeStamp,
) -> u32;

pub type AcquireCredentialsHandleAFunc = unsafe extern "system" fn(
    pszPrincipal: *const i8, // SEC_CHAR
    pszPackage: *const i8,   // SEC_CHAR
    fCredentialUse: u32,
    pvLogonId: *mut c_void,
    pAuthData: *mut c_void,
    pGetKeyFn: *mut c_void,
    pvGetKeyArgument: *mut c_void,
    phCredential: *mut CredHandle,
    ptsExpiry: *mut TimeStamp,
) -> u32;

pub type InitializeSecurityContextWFunc = unsafe extern "system" fn(
    ph_credential: *mut SecHandle,
    ph_context: *mut SecHandle,
    psz_target_name: *mut u16,
    f_context_req: u32,
    reserved1: u32,
    target_data_rep: u32,
    p_input: *mut SecBufferDesc,
    reserved2: u32,
    ph_new_context: *mut SecHandle,
    p_output: *mut SecBufferDesc,
    pf_context_attr: *mut u32,
    pts_expiry: *mut TimeStamp,
) -> u32;

pub type EncryptMessageFunc = unsafe extern "system" fn(
    ph_context: *mut SecHandle,
    f_qop: u32,
    p_message: *mut SecBufferDesc,
    message_seq_no: u32,
) -> u32;

pub type DecryptMessageFunc = unsafe extern "system" fn(
    ph_context: *mut SecHandle,
    p_message: *mut SecBufferDesc,
    message_seq_no: u32,
    pf_qop: *mut u32,
) -> u32;

pub type CompleteAuthTokenFunc =
    unsafe extern "system" fn(phContext: *mut CtxtHandle, pToken: *mut SecBufferDesc) -> i32;

pub type DeleteSecurityContextFunc = unsafe extern "system" fn(phContext: *mut SecHandle) -> u32;
pub type FreeCredentialHandleFunc = unsafe extern "system" fn(phCredential: *mut SecHandle) -> i32;

// Firma della funzione QueryContextAttributesW
pub type QueryContextAttributesWFunc = unsafe extern "system" fn(
    phContext: *mut CtxtHandle,
    ulAttribute: u32,
    pBuffer: *mut c_void,
) -> u32;

pub type FreeContextBufferFunc = unsafe extern "system" fn(pvContextBuffer: *mut c_void) -> i32;

pub struct Sspicli {
    pub acquire_credentials_handle_w: AcquireCredentialsHandleWFunc,
    pub acquire_credentials_handle_a: AcquireCredentialsHandleAFunc,
    pub initialize_security_context_w: InitializeSecurityContextWFunc,
    pub encrypt_message: EncryptMessageFunc,
    pub decrypt_message: DecryptMessageFunc,
    pub complete_auth_token: CompleteAuthTokenFunc,
    pub delete_security_context: DeleteSecurityContextFunc,
    pub free_credential_handle: FreeCredentialHandleFunc,
    pub query_context_attributes_w: QueryContextAttributesWFunc,
    pub free_context_buffer: FreeContextBufferFunc,
}

impl Sspicli {
    pub fn new() -> Self {
        Sspicli {
            acquire_credentials_handle_w: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            acquire_credentials_handle_a: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            initialize_security_context_w: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            encrypt_message: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            decrypt_message: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            complete_auth_token: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            delete_security_context: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            free_credential_handle: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            query_context_attributes_w: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            free_context_buffer: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
        }
    }
}

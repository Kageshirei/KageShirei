use core::ffi::{c_ulong, c_void};

// Constants
pub const X509_ASN_ENCODING: c_ulong = 0x00000001;
pub const PKCS_7_ASN_ENCODING: c_ulong = 0x00010000;
pub const CERT_FIND_SUBJECT_STR_W: c_ulong = 0x00080007;

pub const SP_PROT_TLS1_CLIENT: c_ulong = 0x00000040;
pub const SP_PROT_TLS1_SERVER: c_ulong = 0x00000080;
pub const SP_PROT_TLS1: c_ulong = SP_PROT_TLS1_CLIENT | SP_PROT_TLS1_SERVER;

pub const SP_PROT_TLS1_2_CLIENT: c_ulong = 0x00000800;
pub const SP_PROT_TLS1_2_SERVER: c_ulong = 0x00000400;
pub const SP_PROT_TLS1_2: c_ulong = SP_PROT_TLS1_2_CLIENT | SP_PROT_TLS1_2_SERVER;

#[repr(C)]
pub struct CertContext {
    pub dw_cert_encoding_type: c_ulong,
    pub pb_cert_encoded: *const u8,
    pub cb_cert_encoded: c_ulong,
    pub p_cert_info: *const c_void,
    pub h_cert_store: *const c_void,
}

pub type CertOpenSystemStoreW = unsafe extern "system" fn(
    hProv: *const c_void,
    szSubsystemProtocol: *const u16,
) -> *const c_void;

pub type CertFindCertificateInStore = unsafe extern "system" fn(
    hCertStore: *const c_void,
    dwCertEncodingType: c_ulong,
    dwFindFlags: c_ulong,
    dwFindType: c_ulong,
    pvFindPara: *const c_void,
    pPrevCertContext: *const c_void,
) -> *const CertContext;

pub type CertFreeCertificateContext =
    unsafe extern "system" fn(pCertContext: *const CertContext) -> i32;

pub type CertCloseStore =
    unsafe extern "system" fn(hCertStore: *const c_void, dwFlags: c_ulong) -> i32;

pub struct Crypt32 {
    pub cert_open_system_store_w: CertOpenSystemStoreW,
    pub cert_find_certificate_in_store: CertFindCertificateInStore,
    pub cert_free_certificate_context: CertFreeCertificateContext,
    pub cert_close_store: CertCloseStore,
}

impl Crypt32 {
    pub fn new() -> Self {
        Crypt32 {
            cert_open_system_store_w: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            cert_find_certificate_in_store: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            cert_free_certificate_context: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
            cert_close_store: unsafe {
                core::mem::transmute(core::ptr::null::<core::ffi::c_void>())
            },
        }
    }
}

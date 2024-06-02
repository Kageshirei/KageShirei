use core::error::Error;
use core::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum IndirectSyscallError {
    /// Error loading the PEB (Process Environment Block)
    PebLoadingError,

    /// The PEB Loader Data is null
    NullPebLdrData,

    /// The Ldr (Loader) Flink pointer is null
    NullLdrFlink,

    /// The module base address is not found
    ModuleNotFoundError,

    /// The DOS signature is invalid (expected "MZ")
    InvalidDosSignature,

    /// The NT signature is invalid (expected "PE\0\0")
    InvalidNtSignature,

    /// The export directory pointer is null
    NullExportDirectory,

    /// The function address is not found
    FunctionNotFoundError,
}

impl Display for IndirectSyscallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            IndirectSyscallError::PebLoadingError => write!(f, "Failed to load PEB"),
            IndirectSyscallError::NullPebLdrData => write!(f, "PEB Loader Data is null"),
            IndirectSyscallError::NullLdrFlink => write!(f, "Loader Flink is null"),
            IndirectSyscallError::ModuleNotFoundError => write!(f, "Module not found"),
            IndirectSyscallError::InvalidDosSignature => write!(f, "Invalid DOS Signature"),
            IndirectSyscallError::InvalidNtSignature => write!(f, "Invalid NT Signature"),
            IndirectSyscallError::NullExportDirectory => write!(f, "Export directory is null"),
            IndirectSyscallError::FunctionNotFoundError => write!(f, "Function not found"),
        }
    }
}

impl Error for IndirectSyscallError {}

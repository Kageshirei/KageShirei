use core::error::Error;
use core::ffi::c_ulong;
use core::fmt::{Debug, Display, Formatter};

struct NtdllConfig {
	/// The virtual address of the array of addresses of NTDLL exported functions (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfFunctions]`)
	array_of_addresses: *const u32,
	/// The virtual address of the array of names of NTDLL exported functions (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfNames]`)
	array_of_names: *const u32,
	/// The virtual address of the array of ordinals of NTDLL exported functions (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfNameOrdinals]`)
	array_of_ordinals: *const u16,
	/// The number of exported functions in NTDLL
	number_of_functions: u32,
	/// The base address of NTDLL
	module_base: *const c_ulong,
}

struct PebLoadingError;

impl Debug for PebLoadingError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "Failed to load PEB")
	}
}

impl Display for PebLoadingError {
	fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
		write!(f, "Failed to load PEB")
	}
}

impl Error for PebLoadingError {}


impl NtdllConfig {
	pub unsafe fn instance() -> Result<Self, PebLoadingError> {
		let peb = crate::peb::find_peb();

		if peb.is_null() {
			return Err(PebLoadingError);
		}

		// getting ntdll.dll module (skipping our local image element)
	}
}
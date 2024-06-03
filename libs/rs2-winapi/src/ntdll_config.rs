use crate::errors::LoaderApiError;
use crate::loaderapi::get_module_addr;
use crate::loaderapi::get_nt_headers;
use crate::winnt::ImageExportDirectory;

pub struct NtdllConfig {
    /// The virtual address of the array of addresses of NTDLL exported functions
    /// (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfFunctions]`)
    pub array_of_addresses: *const u32,
    /// The virtual address of the array of names of NTDLL exported functions
    /// (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfNames]`)
    pub array_of_names: *const u32,
    /// The virtual address of the array of ordinals of NTDLL exported functions
    /// (aka `[BaseAddress + IMAGE_EXPORT_DIRECTORY.AddressOfNameOrdinals]`)
    pub array_of_ordinals: *const u16,
    /// The number of exported functions in NTDLL
    pub number_of_functions: u32,
    /// The base address of NTDLL
    pub module_base: *mut u8,
}

/// We implement Sync for NtdllConfig to ensure that it can be safely shared
/// across multiple threads. This is necessary because lazy_static requires
/// the types it manages to be Sync. Since NtdllConfig only contains raw pointers
/// and does not perform any interior mutability, it is safe to implement Sync manually.
unsafe impl Sync for NtdllConfig {}

impl NtdllConfig {
    /// Creates a new instance of `NtdllConfig` by retrieving and validating the necessary NTDLL data.
    ///
    /// This method returns a configured `NtdllConfig` struct containing pointers to the arrays of addresses,
    /// names, and ordinals of NTDLL exported functions, as well as the base address and the number of exported functions.
    ///
    /// # Safety
    ///
    /// This function involves unsafe operations and raw pointers, which require careful handling.
    ///
    /// # Errors
    ///
    /// Returns an `IndirectSyscallError` if there is an issue retrieving the module base address,
    /// NT headers, or if the export directory is null.
    pub unsafe fn instance() -> Result<Self, LoaderApiError> {
        const NTDLL_HASH: u32 = 0x1edab0ed;

        // Get the base address of the NTDLL module
        let module_base = match get_module_addr(NTDLL_HASH) {
            Ok(module_base) => module_base,
            Err(e) => return Err(e),
        };

        // Get the NT headers for the NTDLL module
        let p_img_nt_headers = match get_nt_headers(module_base) {
            Ok(p_img_nt_headers) => p_img_nt_headers,
            Err(e) => return Err(e),
        };

        // Get the export directory from the NT headers
        let data_directory = &(*p_img_nt_headers).optional_header.data_directory[0]; // Assuming IMAGE_DIRECTORY_ENTRY_EXPORT is 0
        let export_directory = (module_base.offset(data_directory.virtual_address as isize))
            as *mut ImageExportDirectory;
        if export_directory.is_null() {
            return Err(LoaderApiError::NullExportDirectory);
        }

        // Initialize the NtdllConfig structure's elements
        let ntdll_config = Self {
            module_base: module_base,
            number_of_functions: (*export_directory).number_of_functions,
            array_of_names: module_base.offset((*export_directory).address_of_names as isize)
                as *const u32,
            array_of_addresses: module_base
                .offset((*export_directory).address_of_functions as isize)
                as *const u32,
            array_of_ordinals: module_base
                .offset((*export_directory).address_of_name_ordinals as isize)
                as *const u16,
        };

        // Validate NtdllConfig elements
        if ntdll_config.module_base.is_null()
            || ntdll_config.number_of_functions == 0
            || ntdll_config.array_of_names.is_null()
            || ntdll_config.array_of_addresses.is_null()
            || ntdll_config.array_of_ordinals.is_null()
        {
            return Err(LoaderApiError::NullExportDirectory); // Consider a more specific error if needed
        } else {
            return Ok(ntdll_config);
        }
    }
}

#[cfg(test)]
mod tests {
    use libc_print::libc_println;

    use super::*;

    #[test]
    fn test_instance() {
        match unsafe { NtdllConfig::instance() } {
            Ok(ntdll_config) => {
                libc_println!("NtdllConfig instance:");

                libc_println!("  Module Base: {:?}", ntdll_config.module_base);
                libc_println!(
                    "  Number of Functions: {:?}",
                    ntdll_config.number_of_functions,
                );
                libc_println!("  Array of Names: {:?}", ntdll_config.array_of_names);
                libc_println!(
                    "  Array of Addresses: {:?}",
                    ntdll_config.array_of_addresses
                );
                libc_println!("  Array of Ordinals: {:?}", ntdll_config.array_of_ordinals);
            }

            Err(err) => {
                libc_println!("Error: {:?}", err);
                assert!(false); // Fail the test if there's an error
            }
        }
    }
}

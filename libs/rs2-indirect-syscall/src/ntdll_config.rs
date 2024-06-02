use crate::errors::IndirectSyscallError;
use crate::headers::{ImageDosHeader, ImageExportDirectory, ImageNtHeaders64};
use crate::headers::{IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE};
use crate::peb::LoaderDataTableEntry;
use crate::peb::PebLoaderData;

use crate::utils::dbj2_hash;

/// Retrieves the base address of a module by its hash.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointers, which require careful handling.
///
/// # Parameters
///
/// - `module_hash`: The hash of the module name.
///
/// # Returns
///
/// The base address of the module as a `*mut u8`, or an `IndirectSyscallError` if the module is not found.
pub unsafe fn get_module_addr(module_hash: u32) -> Result<*mut u8, IndirectSyscallError> {
    // Get the PEB (Process Environment Block)
    let peb = crate::peb::find_peb();

    if peb.is_null() {
        return Err(IndirectSyscallError::PebLoadingError);
    }

    // Get the pointer to the PEB Loader Data
    let peb_ldr_data_ptr = (*peb).loader_data as *mut PebLoaderData;
    if peb_ldr_data_ptr.is_null() {
        return Err(IndirectSyscallError::NullPebLdrData);
    }

    // Get the first module in the InLoadOrderModuleList
    let mut module_list =
        (*peb_ldr_data_ptr).in_load_order_module_list.flink as *mut LoaderDataTableEntry;

    // Iterate through the loaded modules
    while !(*module_list).dll_base.is_null() {
        // Get the DLL name buffer pointer and length
        let dll_buffer_ptr = (*module_list).base_dll_name.buffer;
        let dll_length = (*module_list).base_dll_name.length as usize;
        // Create a slice for the DLL name
        let dll_name_slice = core::slice::from_raw_parts(dll_buffer_ptr as *const u8, dll_length);

        // Check if the hash of the DLL name matches the given hash
        if module_hash == dbj2_hash(dll_name_slice) {
            // Return the base address of the DLL
            return Ok((*module_list).dll_base as _);
        }

        // Move to the next module in the list
        module_list = (*module_list).in_load_order_links.flink as *mut LoaderDataTableEntry;
    }

    // Return an error if the module is not found
    Err(IndirectSyscallError::ModuleNotFoundError)
}

/// Retrieves the NT headers of a module for x86_64 architecture.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointers, which require careful handling.
///
/// # Parameters
///
/// - `base_addr`: The base address of the module.
///
/// # Returns
///
/// A pointer to the NT headers, or an `IndirectSyscallError` if the NT headers are not found or invalid.
#[cfg(target_arch = "x86_64")]
pub unsafe fn get_nt_headers(
    base_addr: *mut u8,
) -> Result<*mut ImageNtHeaders64, IndirectSyscallError> {
    // Cast base_addr to ImageDosHeader pointer
    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err(IndirectSyscallError::InvalidDosSignature);
    }

    // Calculate the address of NT headers
    let nt_headers =
        (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeaders64;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE as _ {
        return Err(IndirectSyscallError::InvalidNtSignature);
    }

    // Return NT headers pointer
    Ok(nt_headers)
}

/// Retrieves the NT headers of a module for x86 architecture.
///
/// # Safety
///
/// This function involves unsafe operations and raw pointers, which require careful handling.
///
/// # Parameters
///
/// - `base_addr`: The base address of the module.
///
/// # Returns
///
/// A pointer to the NT headers, or an `IndirectSyscallError` if the NT headers are not found or invalid.
#[cfg(target_arch = "x86")]
pub unsafe fn get_nt_headers(
    base_addr: *mut u8,
) -> Result<*mut ImageNtHeaders32, IndirectSyscallError> {
    use crate::headers::ImageNtHeaders32;

    // Cast base_addr to ImageDosHeader pointer
    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err(IndirectSyscallError::InvalidDosSignature);
    }

    // Calculate the address of NT headers
    let nt_headers =
        (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeaders32;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE as _ {
        return Err(IndirectSyscallError::InvalidNtSignature);
    }

    // Return NT headers pointer
    Ok(nt_headers)
}

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
    pub unsafe fn instance() -> Result<Self, IndirectSyscallError> {
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
            return Err(IndirectSyscallError::NullExportDirectory);
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
            return Err(IndirectSyscallError::NullExportDirectory); // Consider a more specific error if needed
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

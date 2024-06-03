use crate::{
    errors::LoaderApiError,
    utils::dbj2_hash,
    winnt::{ImageDosHeader, ImageNtHeaders, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE},
    winternl::{find_peb, LoaderDataTableEntry, PebLoaderData},
};

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
/// The base address of the module as a `*mut u8`, or an `LoaderApiError` if the module is not found.
pub unsafe fn get_module_addr(module_hash: u32) -> Result<*mut u8, LoaderApiError> {
    // Get the PEB (Process Environment Block)
    let peb = find_peb();

    if peb.is_null() {
        return Err(LoaderApiError::PebLoadingError);
    }

    // Get the pointer to the PEB Loader Data
    let peb_ldr_data_ptr = (*peb).loader_data as *mut PebLoaderData;
    if peb_ldr_data_ptr.is_null() {
        return Err(LoaderApiError::NullPebLdrData);
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
    Err(LoaderApiError::ModuleNotFoundError)
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
/// A pointer to the NT headers, or an `LoaderApiError` if the NT headers are not found or invalid.
#[cfg(target_arch = "x86_64")]
pub unsafe fn get_nt_headers(base_addr: *mut u8) -> Result<*mut ImageNtHeaders, LoaderApiError> {
    // Cast base_addr to ImageDosHeader pointer

    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err(LoaderApiError::InvalidDosSignature);
    }

    // Calculate the address of NT headers
    let nt_headers = (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeaders;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE as _ {
        return Err(LoaderApiError::InvalidNtSignature);
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
/// A pointer to the NT headers, or an `LoaderApiError` if the NT headers are not found or invalid.
#[cfg(target_arch = "x86")]
pub unsafe fn get_nt_headers(base_addr: *mut u8) -> Result<*mut ImageNtHeaders, LoaderApiError> {
    // Cast base_addr to ImageDosHeader pointer
    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err(LoaderApiError::InvalidDosSignature);
    }

    // Calculate the address of NT headers
    let nt_headers = (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeader;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE as _ {
        return Err(LoaderApiError::InvalidNtSignature);
    }

    // Return NT headers pointer
    Ok(nt_headers)
}

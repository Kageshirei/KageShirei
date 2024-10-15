use core::{arch::asm, ptr::null_mut};

use kageshirei_win32::{
    ntdef::{
        ImageDosHeader,
        ImageExportDirectory,
        ImageNtHeaders,
        LoaderDataTableEntry,
        PebLoaderData,
        HANDLE,
        IMAGE_DOS_SIGNATURE,
        IMAGE_NT_SIGNATURE,
        PEB,
        TEB,
    },
    utils::string_length_a,
};

use crate::common::dbj2_hash;

/// Find the Thread Environment Block (TEB) of the current process on x86_64
#[cfg(target_arch = "x86_64")]
pub fn nt_current_teb() -> *mut TEB {
    let teb_ptr: *mut TEB;
    unsafe {
        asm!(
            "mov {}, gs:[0x30]",
            out(reg) teb_ptr
        );
    }
    teb_ptr
}

/// Find the Thread Environment Block (TEB) of the current process on x86
#[cfg(target_arch = "x86")]
pub fn nt_current_teb() -> *const TEB {
    let teb_ptr: *const TEB;
    unsafe {
        asm!(
            "mov {}, fs:[0x18]",
            out(reg) teb_ptr
        );
    }
    teb_ptr
}

pub fn nt_current_peb() -> *mut PEB { unsafe { nt_current_teb().as_ref().unwrap().process_environment_block } }

/// Gets the last error value for the current thread.
///
/// This function retrieves the last error code set in the Thread Environment Block (TEB).
/// It mimics the behavior of the `NtGetLastError` macro in C.
pub unsafe fn nt_get_last_error() -> u32 { nt_current_teb().as_ref().unwrap().last_error_value }

/// Sets the last error value for the current thread.
///
/// This function sets the last error code in the Thread Environment Block (TEB).
/// It mimics the behavior of the `NtSetLastError` macro in C.
pub unsafe fn nt_set_last_error(error: u32) { nt_current_teb().as_mut().unwrap().last_error_value = error; }

/// Retrieves a handle to the process heap.
///
/// This function returns a handle to the heap used by the process, which is stored in the Process Environment Block
/// (PEB). It mimics the behavior of the `NtProcessHeap` macro in C.
pub unsafe fn nt_process_heap() -> HANDLE {
    nt_current_teb()
        .as_ref()
        .unwrap()
        .process_environment_block
        .as_ref()
        .unwrap()
        .process_heap
}

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
pub unsafe fn ldr_module_peb(module_hash: u32) -> *mut u8 {
    // Get the PEB (Process Environment Block)
    let peb = nt_current_peb();

    if peb.is_null() {
        return null_mut();
    }

    // Get the pointer to the PEB Loader Data
    let peb_ldr_data_ptr = (*peb).loader_data as *mut PebLoaderData;
    if peb_ldr_data_ptr.is_null() {
        return null_mut();
    }

    // Get the first module in the InLoadOrderModuleList
    let mut module_list = (*peb_ldr_data_ptr).in_load_order_module_list.flink as *mut LoaderDataTableEntry;

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
            return (*module_list).dll_base as *mut u8;
        }

        // Move to the next module in the list
        module_list = (*module_list).in_load_order_links.flink as *mut LoaderDataTableEntry;
    }

    // Return an error if the module is not found
    null_mut()
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
pub unsafe fn get_nt_headers(base_addr: *mut u8) -> *mut ImageNtHeaders {
    // Cast base_addr to ImageDosHeader pointer

    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return null_mut();
    }

    // Calculate the address of NT headers
    let nt_headers = (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeaders;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE {
        return null_mut();
    }

    // Return NT headers pointer
    nt_headers
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
pub unsafe fn get_nt_headers(base_addr: *mut u8) -> *mut ImageNtHeaders {
    // Cast base_addr to ImageDosHeader pointer
    let dos_header = base_addr as *mut ImageDosHeader;

    // Check DOS signature (MZ)
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return null_mut();
    }

    // Calculate the address of NT headers
    let nt_headers = (base_addr as isize + (*dos_header).e_lfanew as isize) as *mut ImageNtHeaders;

    // Check NT signature (PE\0\0)
    if (*nt_headers).signature != IMAGE_NT_SIGNATURE as _ {
        return null_mut();
    }

    // Return NT headers pointer
    nt_headers
}

/// Resolves the address of a syscall by searching the export table of NTDLL for the function
/// name that matches the given hash.
///
/// ### Safety
///
/// This function involves unsafe operations and raw pointers, which require careful handling.
///
/// ### Parameters
///
/// - `ntdll_config`: A reference to an `NtdllConfig` struct containing the configuration of NTDLL.
/// - `function_hash`: The hash of the function to be resolved.
///
/// ### Returns
///
/// A pointer to the address of the resolved syscall function. Returns a null pointer if the function is not found.
pub unsafe fn ldr_function_addr(module_base: *mut u8, function_hash: usize) -> *mut u8 {
    // Get the NT headers for the NTDLL module
    let p_img_nt_headers = get_nt_headers(module_base);

    if p_img_nt_headers.is_null() {
        return null_mut();
    }

    // Get the export directory from the NT headers
    let data_directory = &(*p_img_nt_headers).optional_header.data_directory[0]; // Assuming IMAGE_DIRECTORY_ENTRY_EXPORT is 0
    let export_directory = (module_base.offset(data_directory.virtual_address as isize)) as *mut ImageExportDirectory;
    if export_directory.is_null() {
        return null_mut();
    }

    let number_of_functions = (*export_directory).number_of_functions;
    let array_of_names = module_base.offset((*export_directory).address_of_names as isize) as *const u32;
    let array_of_addresses = module_base.offset((*export_directory).address_of_functions as isize) as *const u32;
    let array_of_ordinals = module_base.offset((*export_directory).address_of_name_ordinals as isize) as *const u16;

    // Create a slice from the array of names in the export directory
    let names = core::slice::from_raw_parts(array_of_names, number_of_functions as usize);

    // Create a slice from the array of addresses in the export directory
    let functions = core::slice::from_raw_parts(array_of_addresses, number_of_functions as usize);

    // Create a slice from the array of ordinals in the export directory
    let ordinals = core::slice::from_raw_parts(array_of_ordinals, number_of_functions as usize);

    // Iterate over the names to find the function with the matching hash
    for i in 0 .. number_of_functions {
        // Get the address of the current export name
        let name_addr = module_base.offset(names[i as usize] as isize) as *const i8;
        // Get the length of the C string
        let name_len = string_length_a(name_addr as *const u8);
        // Create a slice for the name
        let name_slice: &[u8] = core::slice::from_raw_parts(name_addr as *const u8, name_len);

        // Check if the hash of the name matches the given hash
        if function_hash as u32 == dbj2_hash(name_slice) {
            // Get the ordinal for the function
            let ordinal = ordinals[i as usize] as usize;
            return module_base.offset(functions[ordinal] as isize) as *mut u8;
        }
    }

    // Return null pointer if function is not found
    null_mut()
}

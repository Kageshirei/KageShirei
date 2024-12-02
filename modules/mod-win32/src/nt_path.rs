use alloc::{boxed::Box, format, vec, vec::Vec};
use core::{ops::Div as _, ptr::null_mut};

use kageshirei_win32::{
    ntdef::{
        IoStatusBlock,
        ObjectAttributes,
        RtlPathType,
        RtlRelativeNameU,
        RtlUserProcessParameters,
        UnicodeString,
        CURDIR,
        FILE_DIRECTORY_FILE,
        FILE_SHARE_READ,
        FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
        FILE_TRAVERSE,
        HANDLE,
        NTSTATUS,
        OBJ_CASE_INSENSITIVE,
        OBJ_INHERIT,
        PWSTR,
        SYNCHRONIZE,
        UNICODE_STRING_MAX_BYTES,
    },
    ntstatus::{NT_SUCCESS, STATUS_NAME_TOO_LONG, STATUS_NO_MEMORY, STATUS_OBJECT_NAME_INVALID, STATUS_SUCCESS},
};
use libc_print::libc_println;
use mod_agentcore::instance;

use crate::{
    nt_peb::get_current_directory,
    utils::{is_path_separator, str_to_unicode_string, NT_STATUS},
};

pub const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

/// Sets the current directory for the calling process.
///
/// This function resolves the provided path to its full NT path and updates the
/// process's current directory to the resolved path. It uses a combination of
/// `rtl_get_full_path_name_ustr`, `rtl_dos_path_name_to_nt_path_name`, and `NtOpenFile`
/// to perform the necessary operations.
///
/// # Parameters
/// - `path`: A string slice representing the directory path to set as the current directory.
///
/// # Returns
/// - `i32`: Returns `STATUS_SUCCESS` on success, or an NTSTATUS error code if the operation fails.
///
/// # Details
/// This function performs the following NT API calls:
/// - `NtOpenFile`: Opens the directory specified by the NT path and obtains a handle to it.
pub fn rtl_set_current_directory(path: &str) -> i32 {
    let mut cur_dir: *mut CURDIR = null_mut();

    unsafe {
        // Retrieve the PEB (Process Environment Block)
        let peb = instance().teb.as_ref().unwrap().process_environment_block;
        if !peb.is_null() {
            // Get the process parameters from the PEB
            let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;

            // Retrieve the current directory
            cur_dir = &mut process_parameters.as_mut().unwrap().current_directory;
        }

        // Obtain the maximum length of the DOS path buffer
        let cur_dir_dos_path_max_length = (*cur_dir).dos_path.maximum_length as usize;

        // Allocate a buffer for the full path, initializing it with zeros
        let mut buffer = vec![0; cur_dir_dos_path_max_length];

        // Initialize a UnicodeString structure to hold the full path
        let mut full_path = UnicodeString {
            length:         0,
            maximum_length: (buffer.capacity().overflowing_mul(2).0) as u16, // Length in bytes
            buffer:         buffer.as_mut_ptr(),
        };

        // Retrieve the full path name using the custom function
        let unicode_path = &str_to_unicode_string(path);

        // let path_utf16_string: Vec<u16> = path.encode_utf16().collect();
        // let mut unicode_path = UnicodeString::new();
        // unicode_path.init(path_utf16_string.as_ptr());

        let full_path_length = rtl_get_full_path_name_ustr(
            &unicode_path,
            full_path.maximum_length as usize,
            full_path.buffer,
            None,
            None,
        );

        // Check for errors during path resolution
        if full_path_length == 0 {
            // libc_println!("[!] Invalid object name: full_path_length <= 0");
            return STATUS_OBJECT_NAME_INVALID; // Invalid object name
        }

        if full_path_length > full_path.maximum_length as usize {
            // libc_println!("[!] Name is too long: full_path_length > maximum_length");
            return STATUS_NAME_TOO_LONG; // Name is too long
        }

        // Update the length of the full_path to the actual resolved length
        full_path.length = full_path_length as u16;

        // Convert the DOS path to an NT path
        let mut nt_name = UnicodeString {
            length:         0,
            maximum_length: full_path.maximum_length,
            buffer:         full_path.buffer,
        };

        if rtlp_win32_nt_name_to_nt_path_name_u(&full_path, &mut nt_name, None, None) != 0 {
            // libc_println!("[!] Invalid object name: rtl_dos_path_name_to_nt_path_name failed");
            return STATUS_OBJECT_NAME_INVALID;
        }

        // Initialize object attributes for the directory we are opening
        let mut object_attributes = ObjectAttributes::new();
        ObjectAttributes::initialize(
            &mut object_attributes,
            &mut nt_name,                       // NT path name
            OBJ_CASE_INSENSITIVE | OBJ_INHERIT, // Flags for case-insensitivity and inheritance
            null_mut(),                         // No root directory
            null_mut(),                         // No security descriptor
        );

        // Prepare for file opening
        let mut status_block = IoStatusBlock::new();
        let mut cur_dir_handle: HANDLE = null_mut();

        // Open the directory
        let status = instance().ntdll.nt_open_file.run(
            &mut cur_dir_handle,
            SYNCHRONIZE | FILE_TRAVERSE,                        // Desired access
            &mut object_attributes,                             // Object attributes
            &mut status_block,                                  // I/O status block
            FILE_SHARE_READ | FILE_SHARE_WRITE,                 // Sharing options
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT, // File options
        );

        // Check if opening the directory succeeded
        if !NT_SUCCESS(status) {
            libc_println!("[!] NtOpenFile failed: {}", NT_STATUS(status));
            return status; // Return the error code
        }

        // Save the new directory handle
        (*cur_dir).handle = cur_dir_handle;

        // Copy the full path into the current directory's DOS path buffer
        core::ptr::copy_nonoverlapping(
            full_path.buffer,
            (*cur_dir).dos_path.buffer,
            (full_path.length.div(2)) as usize, // Copy the length in u16 units
        );

        // Update the DOS path length
        (*cur_dir).dos_path.length = full_path.length;

        0 // Success
    }
}

/// Custom implementation of `RtlGetFullPathName_Ustr` NT API.
///
/// This function determines the full path of a given file name, taking into account
/// the current directory, the type of the path (e.g., absolute, relative, UNC), and
/// possible DOS device names. It is an essential utility for resolving file paths in
/// a Windows-like environment.
///
/// # Parameters
/// - `file_name`: The Unicode string representing the file name.
/// - `size`: The size of the buffer to store the full path.
/// - `buffer`: The buffer to store the full path.
/// - `short_name`: Optionally, the short name part of the path (e.g., file name after the last
///   separator).
/// - `path_type`: Optionally, the type of the path determined by the function.
///
/// # Returns
/// - The size of the full path, or an error code.
///
/// # Safety
/// This function performs unsafe operations, including:
/// - Dereferencing raw pointers (e.g., `buffer`, `prefix`, `source`, etc.).
/// - Modifying the memory location pointed to by `buffer`, `short_name`, and `path_type`.
///
/// The caller must ensure:
/// - The `buffer` pointer is valid and has sufficient memory allocated to hold `size` elements.
/// - The `file_name` parameter points to a valid Unicode string.
/// - The `short_name` and `path_type` parameters, if provided, point to valid memory locations.
/// - The function is used in a context where the Windows process environment is not corrupted.
///
/// Improper use can result in undefined behavior, including memory corruption and program crashes
#[expect(
    unused_assignments,
    reason = "Initial assignments to `prefix_length` and `prefix` are conditionally modified based on the path type"
)]
pub unsafe fn rtl_get_full_path_name_ustr(
    file_name: &UnicodeString,
    size: usize,
    buffer: *mut u16,
    short_name: Option<&mut PWSTR>,
    path_type: Option<&mut RtlPathType>,
) -> usize {
    let file_name_buffer = file_name.buffer;
    let file_name_length = file_name.length as usize;
    let mut prefix_length: usize = 0;
    let mut prefix: *mut u16 = null_mut();
    let mut source = file_name_buffer;
    let mut source_length = file_name_length;

    // Handle initial path type and failure case
    if file_name_length == 0 || *file_name_buffer == 0 {
        return 0;
    }

    // Determine path type
    let path_type_value = rtl_determine_dos_path_name_type_ustr(file_name);
    if let Some(path_type_ref) = path_type {
        *path_type_ref = path_type_value.clone();
    }

    match path_type_value {
        RtlPathType::RtlPathTypeDriveAbsolute => {
            prefix = file_name_buffer;
            prefix_length = 3 * 2; // "C:\"
            source = file_name_buffer.add(3);
            source_length = source_length.overflowing_sub(3 * 2).0;
        },
        RtlPathType::RtlPathTypeRelative | RtlPathType::RtlPathTypeRooted => {
            let mut cur_dir: *mut CURDIR = null_mut();

            let peb = instance().teb.as_ref().unwrap().process_environment_block;
            if !peb.is_null() {
                let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
                cur_dir = &mut process_parameters.as_mut().unwrap().current_directory;
            }

            prefix = (*cur_dir).dos_path.buffer;
            prefix_length = (*cur_dir).dos_path.length as usize;
        },
        RtlPathType::RtlPathTypeUnknown |
        RtlPathType::RtlPathTypeUncAbsolute |
        RtlPathType::RtlPathTypeDriveRelative |
        RtlPathType::RtlPathTypeLocalDevice |
        RtlPathType::RtlPathTypeRootLocalDevice => return 0,
    }

    // Calculate required size
    let (reqsize, _) = prefix_length.overflowing_add(source_length);

    if reqsize > size {
        return reqsize; // Buffer too small, return required size
    }

    // Copy the prefix and the rest of the source path
    if prefix_length > 0 {
        core::ptr::copy_nonoverlapping(prefix, buffer, prefix_length.div(2));
    }
    core::ptr::copy_nonoverlapping(
        source,
        buffer.add(prefix_length.div(2)),
        source_length.div(2),
    );

    // Handle short name (file part)
    if let Some(short_name_ptr) = short_name {
        *short_name_ptr = buffer.add(
            (prefix_length.div(2))
                .overflowing_add(source_length.div(2))
                .0,
        );
    }

    reqsize
}

/// Implementation of the `RtlpDosPathNameToRelativeNtPathName_Ustr` function from the Windows NT
/// API.
///
/// This function converts a given DOS-style path (`dos_file_name`) into its corresponding NT-style
/// path, which is often prefixed with `\\??\\`. The function also supports UNC paths and device
/// paths, and it can extract the "file part" (i.e., the last component of the path) and the
/// "relative name" (i.e., the part of the path relative to the current directory).
///
/// # Parameters
/// - `dos_file_name`: A reference to a `UnicodeString` containing the DOS path to be converted.
/// - `nt_file_name`: A mutable reference to a `UnicodeString` that will be filled with the NT path.
/// - `part_name`: An optional mutable reference to a pointer that will be set to the "file part" of
///   the path (if any).
/// - `relative_name`: An optional mutable reference to a `RtlRelativeNameU` structure that will be
///   filled with the relative name (if applicable).
///
/// # Returns
/// - `NTSTATUS`: Returns `STATUS_SUCCESS` on success, or an appropriate NTSTATUS error code if the
///   operation fails.
///   - `STATUS_OBJECT_NAME_INVALID` if the input path is invalid.
///   - `STATUS_NO_MEMORY` if memory allocation fails.
pub fn rtl_dos_path_name_to_nt_path_name(
    dos_file_name: &UnicodeString,                // Reference to the DOS path to convert
    nt_file_name: &mut UnicodeString,             // Output: NT path
    part_name: Option<&mut *mut u16>,             // Output: Pointer to the file part
    relative_name: Option<&mut RtlRelativeNameU>, // Output: Relative name structure
) -> NTSTATUS {
    unsafe {
        // Define the NT prefix "\\??\\"
        let nt_prefix = str_to_unicode_string("\\??\\");

        // Check if the NT prefix is present in the input path
        let prefix_present = dos_file_name.length > nt_prefix.length as u16 &&
            *dos_file_name.buffer.offset(0) == *nt_prefix.buffer.offset(0) &&
            *dos_file_name.buffer.offset(1) == *nt_prefix.buffer.offset(1) &&
            *dos_file_name.buffer.offset(2) == *nt_prefix.buffer.offset(2) &&
            *dos_file_name.buffer.offset(3) == *nt_prefix.buffer.offset(3);

        if !prefix_present {
            // If the NT prefix is not present, determine the type of the path
            let path_type = rtl_determine_dos_path_name_type_ustr(dos_file_name);

            // Handle different path types and set the prefix parameters
            let (prefix_length, prefix_buffer, prefix_cut) = match path_type {
                RtlPathType::RtlPathTypeUncAbsolute => {
                    let unc_prefix = str_to_unicode_string("\\??\\UNC\\");
                    (unc_prefix.length, unc_prefix.buffer, 2)
                },
                RtlPathType::RtlPathTypeLocalDevice => (nt_prefix.length, nt_prefix.buffer, 4),
                RtlPathType::RtlPathTypeDriveAbsolute |
                RtlPathType::RtlPathTypeDriveRelative |
                RtlPathType::RtlPathTypeRooted |
                RtlPathType::RtlPathTypeRelative => (nt_prefix.length, nt_prefix.buffer, 0),
                RtlPathType::RtlPathTypeUnknown | RtlPathType::RtlPathTypeRootLocalDevice => {
                    return STATUS_OBJECT_NAME_INVALID;
                },
            };

            // Calculate the maximum size of the new path
            let max_length = (dos_file_name.length as usize)
                .overflowing_add(prefix_length as usize)
                .0
                .overflowing_add(2)
                .0
                .div(2);

            let mut new_buffer = vec![0; max_length].into_boxed_slice();

            if new_buffer.is_empty() {
                return STATUS_NO_MEMORY;
            }

            // Copy the prefix into the new buffer
            core::ptr::copy_nonoverlapping(
                prefix_buffer,
                new_buffer.as_mut_ptr(),
                (prefix_length as usize).div(2),
            );

            // Copy the DOS path into the new buffer
            core::ptr::copy_nonoverlapping(
                dos_file_name.buffer.add(prefix_cut),
                new_buffer.as_mut_ptr().add((prefix_length as usize).div(2)),
                ((dos_file_name.length as usize).div(2))
                    .overflowing_sub(prefix_cut)
                    .0,
            );

            // NULL-terminate the new path
            *new_buffer.as_mut_ptr().add(
                ((prefix_length as usize)
                    .div(2)
                    .overflowing_add((dos_file_name.length as usize).div(2))
                    .0)
                    .overflowing_sub(prefix_cut)
                    .0,
            ) = 0;

            // Set the NT file name fields
            nt_file_name.buffer = new_buffer.as_mut_ptr();
            nt_file_name.length = (((prefix_length as usize)
                .overflowing_add(dos_file_name.length as usize)
                .0)
                .overflowing_sub(prefix_cut.overflowing_mul(2).0))
            .0 as u16;
            nt_file_name.maximum_length = max_length as u16;

            // Handle `part_name` if necessary
            if let Some(part_name) = part_name {
                let mut p = new_buffer.as_mut_ptr().add(
                    ((prefix_length as usize)
                        .div(2)
                        .overflowing_add((dos_file_name.length as usize).div(2))
                        .0)
                        .overflowing_sub(prefix_cut)
                        .0,
                );

                // Find the last path separator
                while p > new_buffer.as_mut_ptr() {
                    p = p.sub(1);
                    if *p == OBJ_NAME_PATH_SEPARATOR {
                        p = p.add(1);
                        break;
                    }
                }

                if p > new_buffer.as_mut_ptr() && *p != 0 {
                    *part_name = p;
                }
                else {
                    *part_name = null_mut();
                }
            }

            // Handle `relative_name` if necessary
            if let Some(relative_name) = relative_name {
                let relative_start = new_buffer.as_mut_ptr().add((prefix_length as usize).div(2));

                relative_name.relative_name.buffer = relative_start;
                relative_name.relative_name.length = ((dos_file_name.length as isize)
                    .overflowing_sub(prefix_length as isize)
                    .0) as u16;
                relative_name.relative_name.maximum_length = relative_name.relative_name.length;
                relative_name.containing_directory = null_mut();
                relative_name.cur_dir_ref = null_mut();
            }

            STATUS_SUCCESS
        }
        else {
            // If the NT prefix is already present, use the helper function
            rtlp_win32_nt_name_to_nt_path_name_u(dos_file_name, nt_file_name, part_name, relative_name)
        }
    }
}

// RtlpWin32NTNameToNtPathName_U
/// Implementation of the `RtlpWin32NTNameToNtPathName_U` function from the Windows NT API.
///
/// This function converts a DOS-style path (`dos_path`) into its equivalent NT path (`nt_path`) by
/// handling the appropriate NT prefix (e.g., `\\??\\`). The resulting NT path is compatible with
/// the expected format used internally by the Windows NT kernel and related subsystems.
///
/// # Parameters
/// - `dos_path`: A reference to a `UnicodeString` containing the DOS-style path to be converted.
/// - `nt_path`: A mutable reference to a `UnicodeString` where the resulting NT path will be
///   stored.
/// - `part_name`: An optional mutable reference to a pointer that will be set to the last component
///   of the path (the "file part").
/// - `relative_name`: An optional mutable reference to a `RtlRelativeNameU` structure that will be
///   populated with the relative name, if applicable.
///
/// # Returns
/// - `i32`: Returns `STATUS_SUCCESS` on successful conversion, or an NTSTATUS error code if the
///   operation fails.
pub fn rtlp_win32_nt_name_to_nt_path_name_u(
    dos_path: &UnicodeString,
    nt_path: &mut UnicodeString,
    part_name: Option<&mut *mut u16>,
    relative_name: Option<&mut RtlRelativeNameU>,
) -> i32 {
    unsafe {
        let rtlp_dos_devices_prefix = str_to_unicode_string("\\??\\");

        if dos_path.buffer.is_null() || dos_path.length == 0 {
            return STATUS_OBJECT_NAME_INVALID;
        }

        // Validate the length of the DOS path
        let dos_length = dos_path.length as usize;
        if dos_length >= UNICODE_STRING_MAX_BYTES as usize {
            return STATUS_NAME_TOO_LONG;
        }

        // Determine if the DOS path already starts with the NT prefix
        let nt_prefix_present = dos_length >= rtlp_dos_devices_prefix.length as usize &&
            core::slice::from_raw_parts(
                dos_path.buffer,
                rtlp_dos_devices_prefix.length.div(2) as usize,
            ) == core::slice::from_raw_parts(
                rtlp_dos_devices_prefix.buffer,
                rtlp_dos_devices_prefix.length.div(2) as usize,
            );

        let new_size = if nt_prefix_present {
            dos_length.overflowing_add(2).0 // Just add space for the null terminator
        }
        else {
            dos_length
                .overflowing_add(rtlp_dos_devices_prefix.length as usize)
                .0
                .overflowing_add(2)
                .0
        };

        // let mut new_buffer: Vec<u16> = Vec::with_capacity(new_size / 2);
        // new_buffer.set_len(new_size.div(2));

        let length = new_size.div(2);
        let mut new_buffer = vec![0; length as usize].into_boxed_slice();

        if new_buffer.is_empty() {
            return STATUS_NO_MEMORY;
        }

        if nt_prefix_present {
            // Copy the existing NT-prefixed DOS path into the buffer
            core::ptr::copy_nonoverlapping(dos_path.buffer, new_buffer.as_mut_ptr(), dos_length.div(2));
        }
        else {
            // Copy the NT prefix and the DOS path into the new buffer
            core::ptr::copy_nonoverlapping(
                rtlp_dos_devices_prefix.buffer,
                new_buffer.as_mut_ptr(),
                rtlp_dos_devices_prefix.length.div(2) as usize,
            );

            core::ptr::copy_nonoverlapping(
                dos_path.buffer,
                new_buffer
                    .as_mut_ptr()
                    .add(rtlp_dos_devices_prefix.length.div(2) as usize),
                dos_length.div(2),
            );
        }

        // NULL-terminate the NT path
        *new_buffer
            .as_mut_ptr()
            .add(new_size.div(2).overflowing_sub(1).0) = 0;

        // Handle the relative name if provided
        if let Some(relative_name) = relative_name {
            let relative_name_start = if nt_prefix_present {
                new_buffer.as_mut_ptr()
            }
            else {
                new_buffer
                    .as_mut_ptr()
                    .add(rtlp_dos_devices_prefix.length.div(2) as usize)
            };

            let relative_length = (dos_length as isize)
                .overflowing_sub(
                    if nt_prefix_present {
                        0
                    }
                    else {
                        rtlp_dos_devices_prefix.length as isize
                    },
                )
                .0;

            if relative_length < 0 {
                return STATUS_OBJECT_NAME_INVALID;
            }

            relative_name.relative_name.buffer = relative_name_start;
            relative_name.relative_name.length = relative_length as u16;
            relative_name.relative_name.maximum_length = relative_name.relative_name.length;
            relative_name.containing_directory = null_mut();
            relative_name.cur_dir_ref = null_mut();
        }

        // Handle PartName if provided
        if let Some(part_name) = part_name {
            let mut p = new_buffer.as_mut_ptr().add(
                (dos_length.div(2))
                    .overflowing_add(
                        if nt_prefix_present {
                            0
                        }
                        else {
                            rtlp_dos_devices_prefix.length.div(2) as usize
                        },
                    )
                    .0,
            );

            // Loop from the back until we find a path separator
            while p > new_buffer.as_mut_ptr() {
                p = p.sub(1);
                if *p == OBJ_NAME_PATH_SEPARATOR {
                    p = p.add(1);
                    break;
                }
            }

            if p > new_buffer.as_mut_ptr() && *p != 0 {
                *part_name = p;
            }
            else {
                *part_name = null_mut();
            }
        }

        // Build the final NT path string
        nt_path.buffer = new_buffer.as_mut_ptr();
        nt_path.length = if nt_prefix_present {
            dos_length as u16
        }
        else {
            (dos_length
                .overflowing_add(rtlp_dos_devices_prefix.length as usize)
                .0) as u16
        };
        nt_path.maximum_length = new_size as u16;

        // Prevent Vec from deallocating the buffer when it goes out of scope
        // core::mem::forget(new_buffer);
        Box::leak(new_buffer);

        STATUS_SUCCESS
    }
}

/// Determine the path type of a given Unicode string representing a path.
///
/// This function is an implementation of the `RtlDetermineDosPathNameType_Ustr`
/// from the Windows API. It analyzes the given `UnicodeString` to determine
/// the type of path it represents, such as whether it is a relative path,
/// an absolute path, a UNC path, or a device path.
///
/// # Arguments
/// * `path_string` - A reference to a `UnicodeString` containing the path to analyze.
///
/// # Returns
/// The determined `RtlPathType` indicating the type of the path.
pub fn rtl_determine_dos_path_name_type_ustr(path_string: &UnicodeString) -> RtlPathType {
    let path = path_string.buffer;
    let chars = path_string.length.div(2) as usize;

    // Return RtlPathTypeRelative if there are no characters
    if chars == 0 {
        return RtlPathType::RtlPathTypeRelative;
    }

    // Check if the path starts with a path separator
    if is_path_separator(unsafe { *path }) {
        if chars < 2 || !is_path_separator(unsafe { *path.add(1) }) {
            return RtlPathType::RtlPathTypeRooted; // \x
        }
        if chars < 3 || (unsafe { *path.add(2) } != '.' as u16 && unsafe { *path.add(2) } != '?' as u16) {
            return RtlPathType::RtlPathTypeUncAbsolute; // \\x
        }
        if chars >= 4 && is_path_separator(unsafe { *path.add(3) }) {
            return RtlPathType::RtlPathTypeLocalDevice; // \\.\x or \\?\x
        }
        if chars == 3 {
            return RtlPathType::RtlPathTypeRootLocalDevice; // \\. or \\?
        }
        RtlPathType::RtlPathTypeUncAbsolute // \\.x or \\?x
    }
    else {
        if chars < 2 || unsafe { *path.add(1) } != ':' as u16 {
            return RtlPathType::RtlPathTypeRelative; // x
        }
        if chars < 3 || !is_path_separator(unsafe { *path.add(2) }) {
            return RtlPathType::RtlPathTypeDriveRelative; // x:
        }
        RtlPathType::RtlPathTypeDriveAbsolute // x:\
    }
}

/// Changes the current directory to the parent directory (handles "cd ..").
///
/// This function moves up one level in the directory hierarchy by adjusting the current
/// directory string. It retrieves the current directory using the process's environment
/// block (PEB) and then modifies the string to remove the last directory component.
/// Finally, it calls `rtl_set_current_directory` to update the current directory.
///
/// # Returns
/// - `i32`: Returns `STATUS_SUCCESS` on success, or an NTSTATUS error code if the operation fails.
///
/// # Details
/// This function directly manipulates the current directory string and relies on the
/// `rtl_set_current_directory` function to apply the changes. It is designed to handle
/// cases where the current directory is the root of a drive (e.g., "C:\").
pub fn change_to_parent_directory() -> i32 {
    // Convert the current directory's DOS path to a Rust string
    let current_directory_str = get_current_directory();

    if current_directory_str.is_empty() {
        return -1;
    }
    // Find the position of the last path separator
    if let Some(parent_end) = current_directory_str.rfind(char::from_u32(OBJ_NAME_PATH_SEPARATOR as u32).unwrap()) {
        let mut parent_directory = &current_directory_str[.. parent_end];

        // Handle case where the parent directory is the root of the drive (e.g., "C:")
        let formatted_directory;
        if parent_directory.ends_with(':') {
            formatted_directory = format!("{}\\", parent_directory);
            parent_directory = &formatted_directory;
        }

        // Set the current directory to the parent directory
        return rtl_set_current_directory(parent_directory);
    }

    STATUS_OBJECT_NAME_INVALID
}

/// Changes the current directory to the specified path.
///
/// This function handles both absolute and relative paths, including support for multiple
/// parent directory navigations using `".."`. If the path contains `".."`, the function
/// will split the path and handle each `".."` separately by moving up one directory level
/// using the `change_to_parent_directory` function. For all other components of the path,
/// it attempts to change the directory using the `rtl_set_current_directory` function,
/// which interfaces with the underlying NT API.
///
/// # Parameters
/// - `path`: A string slice representing the directory path to change to. This can include multiple
///   `".."` components to move up multiple levels in the directory hierarchy.
///
/// # Returns
/// - `i32`: Returns `STATUS_SUCCESS` on success, or an NTSTATUS error code if the operation fails.
///
/// # Details
/// - If the path contains `".."`, the function will handle each `".."` sequentially, moving up one
///   directory level for each occurrence.
/// - For simple absolute or relative paths without `".."`, the function directly uses
///   `rtl_set_current_directory` to change the directory.
/// - This function is useful in low-level environments where direct manipulation of the process's
///   current directory is required.
pub fn change_directory(path: &str) -> i32 {
    if path.contains("..") {
        // Split the path into components
        let parts: Vec<&str> = path.split('\\').collect();
        let mut current_status = STATUS_SUCCESS;

        for part in parts {
            if part == ".." {
                // Go up one directory level
                current_status = change_to_parent_directory();
                if current_status != STATUS_SUCCESS {
                    return current_status; // Return error if any part fails
                }
            }
            else if !part.is_empty() && part != "." {
                // Attempt to change to the specified directory part
                current_status = rtl_set_current_directory(part);
                if current_status != STATUS_SUCCESS {
                    return current_status; // Return error if any part fails
                }
            }
        }

        return current_status;
    }

    // Handle cases where path doesn't contain ".." or "."
    rtl_set_current_directory(path)
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::ptr::null_mut;

    use libc_print::libc_println;

    use super::*;
    use crate::{
        nt_peb::get_current_directory,
        utils::{ptr_to_str, unicodestring_to_string},
    };

    #[test]
    #[ignore]
    fn test_rtl_dos_path_name_to_nt_path_name() {
        let mut test_cases = Vec::new();

        test_cases.push((
            str_to_unicode_string("\\??\\C:\\Windows\\file.txt"),
            "\\??\\C:\\Windows\\file.txt",
            RtlPathType::RtlPathTypeRelative,
        ));

        // Test case 1: UNC absolute path
        test_cases.push((
            str_to_unicode_string("\\\\server\\share\\file.txt"),
            "\\??\\UNC\\server\\share\\file.txt",
            RtlPathType::RtlPathTypeUncAbsolute,
        ));

        // Test case 2: Local device path
        test_cases.push((
            str_to_unicode_string("\\\\.\\C:\\file.txt"),
            "\\??\\C:\\file.txt",
            RtlPathType::RtlPathTypeLocalDevice,
        ));

        // Test case 3: Drive absolute path
        test_cases.push((
            str_to_unicode_string("C:\\Windows\\System32"),
            "\\??\\C:\\Windows\\System32",
            RtlPathType::RtlPathTypeDriveAbsolute,
        ));

        // Test case 4: Drive relative path
        test_cases.push((
            str_to_unicode_string("C:file.txt"),
            "\\??\\C:file.txt",
            RtlPathType::RtlPathTypeDriveRelative,
        ));

        // Test case 5: Rooted path
        test_cases.push((
            str_to_unicode_string("\\Windows\\System32\\file.txt"),
            "\\??\\\\Windows\\System32\\file.txt",
            RtlPathType::RtlPathTypeRooted,
        ));

        // Test case 6: Relative path
        test_cases.push((
            str_to_unicode_string("file.txt"),
            "\\??\\file.txt",
            RtlPathType::RtlPathTypeRelative,
        ));

        for (input, expected_nt_path, path_type) in test_cases {
            // let original_path_type = rtl_determine_dos_path_name_type_ustr(&input);

            let mut nt_path = UnicodeString::new();
            let mut part_name: *mut u16 = null_mut();
            let mut relative_name = RtlRelativeNameU::new();

            libc_println!(
                "\n\nTesting path: {:?}",
                unicodestring_to_string(&input).unwrap()
            );

            let status = rtl_dos_path_name_to_nt_path_name(
                &input,
                &mut nt_path,
                Some(&mut part_name),
                Some(&mut relative_name),
            );

            // Verifica che la funzione abbia avuto successo
            libc_println!("Status: 0x{:X}", status);
            assert_eq!(
                status,
                STATUS_SUCCESS,
                "Unexpected status for path: {:?}",
                unicodestring_to_string(&input).unwrap()
            );

            // Verifica che il percorso NT risultante sia corretto
            let nt_path_str = unicodestring_to_string(&nt_path).unwrap();
            libc_println!("Expected NT Path: {}", expected_nt_path);
            libc_println!("Result NT Path:   {}", nt_path_str);
            assert_eq!(
                nt_path_str,
                expected_nt_path,
                "Mismatch in NT Path for input: {:?}",
                unicodestring_to_string(&input).unwrap()
            );

            // Verifica che part_name e relative_name siano impostati correttamente
            if let Some(part_name_ptr) = unsafe { part_name.as_mut() } {
                let part_name_str = unsafe { ptr_to_str(part_name_ptr) };
                libc_println!("Part Name: {}", part_name_str);
                assert!(
                    !part_name_str.is_empty(),
                    "PartName was expected but is empty"
                );
            }
            else {
                libc_println!("Part Name is null");
            }

            // Se il percorso era relativo, verifica che relative_name sia corretto
            if path_type as i32 == RtlPathType::RtlPathTypeRelative as i32 {
                libc_println!(
                    "Relative Name Buffer: {:?}",
                    relative_name.relative_name.buffer
                );
                assert!(
                    !relative_name.relative_name.buffer.is_null(),
                    "RelativeName should not be null for relative paths"
                );
            }

            libc_println!(
                "Test passed for: {:?}",
                unicodestring_to_string(&input).unwrap()
            );
        }
    }

    #[test]
    #[ignore]
    fn test_determine_path_type() {
        let mut test_cases = Vec::new();
        test_cases.push((str_to_unicode_string("\\"), RtlPathType::RtlPathTypeRooted));
        test_cases.push((
            str_to_unicode_string("\\\\"),
            RtlPathType::RtlPathTypeUncAbsolute,
        ));
        test_cases.push((
            str_to_unicode_string("\\\\?\\C:\\path"),
            RtlPathType::RtlPathTypeLocalDevice,
        ));
        test_cases.push((
            str_to_unicode_string("\\\\.\\C:\\path"),
            RtlPathType::RtlPathTypeLocalDevice,
        ));
        test_cases.push((
            str_to_unicode_string("C:\\Windows\\System32"),
            RtlPathType::RtlPathTypeDriveAbsolute,
        ));
        test_cases.push((
            str_to_unicode_string("C:"),
            RtlPathType::RtlPathTypeDriveRelative,
        ));
        test_cases.push((
            str_to_unicode_string("path"),
            RtlPathType::RtlPathTypeRelative,
        ));

        for (input, expected) in test_cases {
            let result = rtl_determine_dos_path_name_type_ustr(&input);
            assert_eq!(
                result as i32,
                expected as i32,
                "Failed for input: {:?}",
                unicodestring_to_string(&input)
            );
        }
    }

    #[test]
    #[ignore]
    fn test_dos_path_to_nt_path() {
        libc_println!("Test 1: Basic DOS path to NT path conversion");
        let dos_path = str_to_unicode_string("C:\\Windows\\System32");
        let mut nt_path = UnicodeString::new();
        let mut part_name: *mut u16 = null_mut();
        let status = rtl_dos_path_name_to_nt_path_name(&dos_path, &mut nt_path, Some(&mut part_name), None);

        assert_eq!(status, STATUS_SUCCESS);
        let nt_path_str = unicodestring_to_string(&nt_path).unwrap();
        libc_println!("NT Path: {}", nt_path_str);
        assert_eq!(nt_path_str, "\\??\\C:\\Windows\\System32");

        if !part_name.is_null() {
            let part_name_str = unsafe { ptr_to_str(part_name) };
            libc_println!("Part Name: {}", part_name_str);
            assert_eq!(part_name_str, "System32");
        }
        else {
            assert!(false, "PartName was expected but is null");
        }
    }

    #[test]
    #[ignore]
    fn test_dos_path_to_nt_path_ending_with_separator() {
        libc_println!("Test 2: DOS path ending with a separator");
        let dos_path = str_to_unicode_string("C:\\Windows\\System32\\");
        let mut nt_path = UnicodeString::new();
        let mut part_name: *mut u16 = null_mut();
        let status = rtl_dos_path_name_to_nt_path_name(&dos_path, &mut nt_path, Some(&mut part_name), None);

        assert_eq!(status, STATUS_SUCCESS);
        let nt_path_str = unicodestring_to_string(&nt_path).unwrap();
        libc_println!("NT Path: {}", nt_path_str);
        assert_eq!(nt_path_str, "\\??\\C:\\Windows\\System32\\");

        // Since the path ends with a separator, part_name should be null
        assert!(
            part_name.is_null(),
            "PartName was not expected but is non-null"
        );
    }

    #[test]
    #[ignore]
    fn test_dos_path_to_nt_path_with_relative_name() {
        libc_println!("Test 3: DOS path with a relative name");
        let dos_path = str_to_unicode_string("\\??\\C:\\Windows\\System32\\file.txt");
        let mut nt_path: UnicodeString = UnicodeString::new();
        let mut part_name: *mut u16 = null_mut();
        let mut relative_name = RtlRelativeNameU::new();
        let status = rtl_dos_path_name_to_nt_path_name(
            &dos_path,
            &mut nt_path,
            Some(&mut part_name),
            Some(&mut relative_name),
        );

        assert_eq!(status, STATUS_SUCCESS);
        let nt_path_str = unicodestring_to_string(&nt_path).unwrap();
        libc_println!("NT Path: {}", nt_path_str);
        assert_eq!(nt_path_str, "\\??\\C:\\Windows\\System32\\file.txt");

        if !part_name.is_null() {
            let part_name_str = unsafe { ptr_to_str(part_name) };
            libc_println!("Part Name: {}", part_name_str);
            assert_eq!(part_name_str, "file.txt");
        }
        else {
            assert!(false, "PartName was expected but is null");
        }
    }

    #[test]
    #[ignore]
    fn test_rtl_get_full_path_name() {
        unsafe {
            // Test 1: Resolve a relative path to an absolute path
            let relative_path = &str_to_unicode_string("Folder\\File.txt");
            let mut buffer: Vec<u16> = Vec::with_capacity(512);
            buffer.set_len(512);
            let mut full_path = UnicodeString {
                length:         0,
                maximum_length: (buffer.capacity() * 2) as u16,
                buffer:         buffer.as_mut_ptr(),
            };

            let full_path_length = rtl_get_full_path_name_ustr(
                relative_path,
                full_path.maximum_length as usize,
                full_path.buffer,
                None,
                None,
            );

            libc_println!("Test 1: Relative Path");
            libc_println!(
                "Relative Path: {}",
                unicodestring_to_string(relative_path).unwrap()
            );
            if full_path_length > 0 {
                full_path.length = full_path_length as u16;
                libc_println!(
                    "Full Path: {}",
                    unicodestring_to_string(&full_path).unwrap()
                );
            }
            else {
                libc_println!("Failed to get the full path");
            }

            // Test 2: Resolve an absolute path
            let absolute_path = &str_to_unicode_string("C:\\Windows\\System32\\cmd.exe");
            let mut buffer: Vec<u16> = Vec::with_capacity(512);
            buffer.set_len(512);
            let mut full_path = UnicodeString {
                length:         0,
                maximum_length: (buffer.capacity() * 2) as u16,
                buffer:         buffer.as_mut_ptr(),
            };

            let full_path_length = rtl_get_full_path_name_ustr(
                absolute_path,
                full_path.maximum_length as usize,
                full_path.buffer,
                None,
                None,
            );

            libc_println!("Test 2: Absolute Path");
            libc_println!(
                "Absolute Path: {:?}",
                unicodestring_to_string(absolute_path).unwrap()
            );
            if full_path_length > 0 {
                full_path.length = full_path_length as u16;
                libc_println!(
                    "Full Path: {}",
                    unicodestring_to_string(&full_path).unwrap()
                );
            }
            else {
                libc_println!("Failed to get the full path");
            }

            // Test 3: Resolve a path with an invalid format
            let invalid_path = &str_to_unicode_string("::InvalidPath::");
            let mut buffer: Vec<u16> = Vec::with_capacity(512);
            buffer.set_len(512);
            let mut full_path = UnicodeString {
                length:         0,
                maximum_length: (buffer.capacity() * 2) as u16,
                buffer:         buffer.as_mut_ptr(),
            };

            let full_path_length = rtl_get_full_path_name_ustr(
                invalid_path,
                full_path.maximum_length as usize,
                full_path.buffer,
                None,
                None,
            );

            libc_println!("Test 3: Invalid Path");
            libc_println!(
                "Invalid Path: {:?}",
                unicodestring_to_string(invalid_path).unwrap()
            );
            if full_path_length > 0 {
                full_path.length = full_path_length as u16;
                libc_println!(
                    "Full Path: {}",
                    unicodestring_to_string(&full_path).unwrap()
                );
            }
            else {
                libc_println!("Failed to get the full path");
            }

            // Test 4: Empty path
            let empty_path = &str_to_unicode_string("");
            let mut buffer: Vec<u16> = Vec::with_capacity(512);
            buffer.set_len(512);
            let mut full_path = UnicodeString {
                length:         0,
                maximum_length: (buffer.capacity() * 2) as u16,
                buffer:         buffer.as_mut_ptr(),
            };

            let full_path_length = rtl_get_full_path_name_ustr(
                empty_path,
                full_path.maximum_length as usize,
                full_path.buffer,
                None,
                None,
            );

            libc_println!("Test 4: Empty Path");
            if full_path_length > 0 {
                full_path.length = full_path_length as u16;
                libc_println!(
                    "Full Path: {}",
                    unicodestring_to_string(&full_path).unwrap()
                );
            }
            else {
                libc_println!("Failed to get the full path");
            }
        }
    }

    #[test]
    #[ignore]
    fn test_change_directory() {
        // Test changing to a subdirectory
        let target_directory = "C:\\Windows\\System32\\drivers\\etc";
        let result = change_directory(target_directory);
        assert_eq!(
            result, 0,
            "Failed to change to directory: {}",
            target_directory
        );

        // Verify that the directory has changed
        let current_directory = get_current_directory();
        assert!(
            current_directory.ends_with(target_directory),
            "Expected directory: {}, but got: {}",
            target_directory,
            current_directory
        );

        // Test changing to the parent directory with "cd .."
        let result = change_directory("..");
        assert_eq!(result, 0, "Failed to change to parent directory");

        // Verify that the directory has moved up one level
        let parent_directory = "C:\\";
        let current_directory = get_current_directory();
        assert!(
            current_directory.starts_with(parent_directory),
            "Expected parent directory: {}, but got: {}",
            parent_directory,
            current_directory
        );

        // Test changing to a subdirectory
        let target_directory = "C:\\Windows\\System32\\drivers";
        let result = change_directory(target_directory);
        assert_eq!(
            result, 0,
            "Failed to change to directory: {}",
            target_directory
        );

        // Verify that the directory has changed
        let current_directory = get_current_directory();
        assert!(
            current_directory.ends_with(target_directory),
            "Expected directory: {}, but got: {}",
            target_directory,
            current_directory
        );

        // Test changing to the parent directory with "cd .."
        let result = change_directory("..\\..");
        assert_eq!(result, 0, "Failed to change to parent directory");

        // Verify that the directory has moved up one level
        let parent_directory = "C:\\Windows";
        let current_directory = get_current_directory();
        assert!(
            current_directory.starts_with(parent_directory),
            "Expected parent directory: {}, but got: {}",
            parent_directory,
            current_directory
        );
    }

    #[test]
    #[ignore]
    fn test_rtl_set_current_directory_w() {
        // Define the target directory for the test
        let target_directory = "C:\\Windows";

        // Call the function to set the current directory
        let result = rtl_set_current_directory(target_directory);

        // Verify the function returned success
        assert_eq!(
            result, 0,
            "Failed to set the current directory to '{}'",
            target_directory
        );

        // Retrieve the current directory to verify if it was actually changed
        let current_directory = unsafe {
            let peb = instance().teb.as_ref().unwrap().process_environment_block;
            if !peb.is_null() {
                let process_parameters = (*peb).process_parameters as *mut RtlUserProcessParameters;
                let cur_dir = &mut process_parameters.as_mut().unwrap().current_directory;
                unicodestring_to_string(&(*cur_dir).dos_path)
            }
            else {
                None
            }
        };

        // Check if the current directory matches the target directory
        match current_directory {
            Some(current_dir) => {
                assert!(
                    current_dir.ends_with(target_directory),
                    "Expected current directory to be '{}', but got '{}'",
                    target_directory,
                    current_dir
                );
            },
            None => {
                panic!("Failed to retrieve the current directory after setting it.");
            },
        }
    }
}

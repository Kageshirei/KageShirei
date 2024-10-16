#![no_std]
/// A crate for detecting syscall numbers by scanning memory for specific assembly patterns.
/// Supports Hell's Gate, Halo's Gate, and Tartarus' Gate techniques for identifying syscalls.
/// Works in `no_std` environments with raw pointer manipulation for high-performance use cases.

/// The offset to move up in memory.
const UP: isize = -32;
/// The offset to move down in memory.
const DOWN: usize = 32;

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
/// The syscall number as a `u16`. Returns 0 if the address is null or the syscall number cannot be
/// determined.
pub unsafe fn get_syscall_number(address: *mut u8) -> u16 {
    // Check if address is null
    if address.is_null() {
        return 0;
    }

    // Hell's gate
    // check if the assembly instruction are:
    // mov r10, rcx
    // mov rcx, <syscall>
    if address.read() == 0x4c &&
        address.add(1).read() == 0x8b &&
        address.add(2).read() == 0xd1 &&
        address.add(3).read() == 0xb8 &&
        address.add(6).read() == 0x00 &&
        address.add(7).read() == 0x00
    {
        let high = address.add(5).read() as u16;
        let low = address.add(4).read() as u16;
        return (high << 8) | low;
    }

    // Halo's Gate Patch
    if address.read() == 0xe9 {
        for idx in 1usize .. 500 {
            let down_offset = idx.wrapping_mul(DOWN);
            let up_offset = (idx as isize).wrapping_mul(UP);

            let total_down_offset_1 = 1usize.wrapping_add(down_offset);
            let total_down_offset_2 = 2usize.wrapping_add(down_offset);
            let total_down_offset_3 = 3usize.wrapping_add(down_offset);
            let total_down_offset_6 = 6usize.wrapping_add(down_offset);
            let total_down_offset_7 = 7usize.wrapping_add(down_offset);

            // if hooked check the neighborhood to find clean syscall (downwards)
            if address.add(down_offset).read() == 0x4c &&
                address.add(total_down_offset_1).read() == 0x8b &&
                address.add(total_down_offset_2).read() == 0xd1 &&
                address.add(total_down_offset_3).read() == 0xb8 &&
                address.add(total_down_offset_6).read() == 0x00 &&
                address.add(total_down_offset_7).read() == 0x00
            {
                let total_high_down_offset = 5usize.wrapping_add(down_offset);
                let total_low_down_offset = 4usize.wrapping_add(down_offset);
                let high = address.add(total_high_down_offset).read() as u16;
                let low = address.add(total_low_down_offset).read() as u16;
                return (high << 8) | low.wrapping_sub(idx as u16);
            }

            let total_up_offset_1 = 1isize.wrapping_add(up_offset);
            let total_up_offset_2 = 2isize.wrapping_add(up_offset);
            let total_up_offset_3 = 3isize.wrapping_add(up_offset);
            let total_up_offset_6 = 6isize.wrapping_add(up_offset);
            let total_up_offset_7 = 7isize.wrapping_add(up_offset);

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(up_offset).read() == 0x4c &&
                address.offset(total_up_offset_1).read() == 0x8b &&
                address.offset(total_up_offset_2).read() == 0xd1 &&
                address.offset(total_up_offset_3).read() == 0xb8 &&
                address.offset(total_up_offset_6).read() == 0x00 &&
                address.offset(total_up_offset_7).read() == 0x00
            {
                let total_high_up_offset = 5isize.wrapping_add(up_offset);
                let total_low_up_offset = 4isize.wrapping_add(up_offset);
                let high = address.offset(total_high_up_offset).read() as u16;
                let low = address.offset(total_low_up_offset).read() as u16;
                return (high << 8) | low.wrapping_add(idx as u16);
            }
        }
    }

    // Tartarus' Gate Patch
    if address.add(3).read() == 0xe9 {
        for idx in 1usize .. 500 {
            let down_offset = idx.wrapping_mul(DOWN);
            let up_offset = (idx as isize).wrapping_mul(UP);

            let total_down_offset_1 = 1usize.wrapping_add(down_offset);
            let total_down_offset_2 = 2usize.wrapping_add(down_offset);
            let total_down_offset_3 = 3usize.wrapping_add(down_offset);
            let total_down_offset_6 = 6usize.wrapping_add(down_offset);
            let total_down_offset_7 = 7usize.wrapping_add(down_offset);

            // if hooked check the neighborhood to find clean syscall (downwards)
            if address.add(down_offset).read() == 0x4c &&
                address.add(total_down_offset_1).read() == 0x8b &&
                address.add(total_down_offset_2).read() == 0xd1 &&
                address.add(total_down_offset_3).read() == 0xb8 &&
                address.add(total_down_offset_6).read() == 0x00 &&
                address.add(total_down_offset_7).read() == 0x00
            {
                let total_high_down_offset = 5usize.wrapping_add(down_offset);
                let total_low_down_offset = 4usize.wrapping_add(down_offset);
                let high = address.add(total_high_down_offset).read() as u16;
                let low = address.add(total_low_down_offset).read() as u16;
                return (high << 8) | low.wrapping_sub(idx as u16);
            }

            let total_up_offset_1 = 1isize.wrapping_add(up_offset);
            let total_up_offset_2 = 2isize.wrapping_add(up_offset);
            let total_up_offset_3 = 3isize.wrapping_add(up_offset);
            let total_up_offset_6 = 6isize.wrapping_add(up_offset);
            let total_up_offset_7 = 7isize.wrapping_add(up_offset);

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(up_offset).read() == 0x4c &&
                address.offset(total_up_offset_1).read() == 0x8b &&
                address.offset(total_up_offset_2).read() == 0xd1 &&
                address.offset(total_up_offset_3).read() == 0xb8 &&
                address.offset(total_up_offset_6).read() == 0x00 &&
                address.offset(total_up_offset_7).read() == 0x00
            {
                let total_high_up_offset = 5isize.wrapping_add(up_offset);
                let total_low_up_offset = 4isize.wrapping_add(up_offset);
                let high = address.offset(total_high_up_offset).read() as u16;
                let low = address.offset(total_low_up_offset).read() as u16;
                return (high << 8) | low.wrapping_add(idx as u16);
            }
        }
    }

    0
}

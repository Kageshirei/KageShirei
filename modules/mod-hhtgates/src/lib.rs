#![no_std]

const UP: isize = -32;
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
        let high = address.add(5).read() as u16;
        let low = address.add(4).read() as u16;
        return ((high << 8) | low) as u16;
        // return ((high.overflowing_shl(8).0) | low) as u16;
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
                let high = address.add(5 + idx * DOWN).read() as u16;
                let low = address.add(4 + idx * DOWN).read() as u16;
                return (high << 8) | (low.wrapping_sub(idx as u16));

                // return ((high.overflowing_shl(8).0) | low - idx as u8) as u16;
            }

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(idx as isize * UP).read() == 0x4c
                && address.offset(1 + idx as isize * UP).read() == 0x8b
                && address.offset(2 + idx as isize * UP).read() == 0xd1
                && address.offset(3 + idx as isize * UP).read() == 0xb8
                && address.offset(6 + idx as isize * UP).read() == 0x00
                && address.offset(7 + idx as isize * UP).read() == 0x00
            {
                let high = address.offset(5 + idx as isize * UP).read() as u16;
                let low = address.offset(4 + idx as isize * UP).read() as u16;
                return (high << 8) | (low.wrapping_add(idx as u16));

                // return ((high.overflowing_shl(8).0) | low + idx as u8) as u16;
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
                // let high: u8 = address.add(5 + idx * DOWN).read();
                // let low: u8 = address.add(4 + idx * DOWN).read();
                // return ((high.overflowing_shl(8).0) | low - idx as u8) as u16;

                let high = address.add(5 + idx * DOWN).read() as u16;
                let low = address.add(4 + idx * DOWN).read() as u16;
                return (high << 8) | (low.wrapping_sub(idx as u16));
            }

            // if hooked check the neighborhood to find clean syscall (upwards)
            if address.offset(idx as isize * UP).read() == 0x4c
                && address.offset(1 + idx as isize * UP).read() == 0x8b
                && address.offset(2 + idx as isize * UP).read() == 0xd1
                && address.offset(3 + idx as isize * UP).read() == 0xb8
                && address.offset(6 + idx as isize * UP).read() == 0x00
                && address.offset(7 + idx as isize * UP).read() == 0x00
            {
                // let high: u8 = address.offset(5 + idx as isize * UP).read();
                // let low: u8 = address.offset(4 + idx as isize * UP).read();
                // return ((high.overflowing_shl(8).0) | low + idx as u8) as u16;

                let high = address.offset(5 + idx as isize * UP).read() as u16;
                let low = address.offset(4 + idx as isize * UP).read() as u16;
                return (high << 8) | (low.wrapping_add(idx as u16));
            }
        }
    }

    return 0;
}

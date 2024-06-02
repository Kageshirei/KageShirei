/// Generates a unique hash using the dbj2 algorithm.
///
/// # Parameters
///
/// - `buffer`: A slice of bytes to hash.
///
/// # Returns
///
/// The hash of the buffer as a `u32`.
pub fn dbj2_hash(buffer: &[u8]) -> u32 {
    let mut hsh: u32 = 5381;
    let mut iter: usize = 0;
    let mut cur: u8;

    while iter < buffer.len() {
        cur = buffer[iter];
        if cur == 0 {
            iter += 1;
            continue;
        }
        if cur >= ('a' as u8) {
            cur -= 0x20;
        }
        hsh = ((hsh << 5).wrapping_add(hsh)) + cur as u32;
        iter += 1;
    }
    hsh
}

/// Gets the length of a C String.
///
/// # Parameters
///
/// - `pointer`: A pointer to the start of the C string.
///
/// # Returns
///
/// The length of the C string as a `usize`.
pub fn get_cstr_len(pointer: *const char) -> usize {
    let mut tmp: u64 = pointer as u64;

    unsafe {
        while *(tmp as *const u8) != 0 {
            tmp += 1;
        }
    }
    (tmp - pointer as u64) as _
}

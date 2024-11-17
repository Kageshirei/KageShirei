use alloc::vec::Vec;

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
        cur = *buffer.get(iter).unwrap_or(&0);
        if cur == 0 {
            iter = iter.overflowing_add(1).0;
            continue;
        }
        if cur >= b'a' {
            cur = cur.overflowing_sub(0x20).0;
        }
        hsh = ((hsh << 5).overflowing_add(hsh).0)
            .overflowing_add(cur as u32)
            .0;
        iter = iter.overflowing_add(1).0;
    }
    hsh
}

// Helper function to create a wide string (UTF-16 encoded)
pub fn wide_string(s: &str) -> Vec<u16> {
    let mut vec: Vec<u16> = s.encode_utf16().collect();
    vec.push(0); // Null-terminate the string
    vec
}

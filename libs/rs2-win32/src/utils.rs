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

pub fn string_length_a(string: *const u8) -> usize {
    unsafe {
        let mut string2 = string;
        while !(*string2).is_null() {
            string2 = string2.add(1);
        }
        string2.offset_from(string) as usize
    }
}

pub fn string_length_w(string: *const u16) -> usize {
    unsafe {
        let mut string2 = string;
        while !(*string2).is_null() {
            string2 = string2.add(1);
        }
        string2.offset_from(string) as usize
    }
}

// Utility function for checking null terminator for u8 and u16
trait IsNull {
    fn is_null(&self) -> bool;
}

impl IsNull for u8 {
    fn is_null(&self) -> bool {
        *self == 0
    }
}

impl IsNull for u16 {
    fn is_null(&self) -> bool {
        *self == 0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_string_length_a() {
        let string = b"hello\0";
        let length = string_length_a(string.as_ptr());
        assert_eq!(length, 5);
    }

    #[test]
    fn test_string_length_w() {
        let string: [u16; 6] = [
            b'h' as u16,
            b'e' as u16,
            b'l' as u16,
            b'l' as u16,
            b'o' as u16,
            0,
        ];
        let length = string_length_w(string.as_ptr());
        assert_eq!(length, 5);
    }
}

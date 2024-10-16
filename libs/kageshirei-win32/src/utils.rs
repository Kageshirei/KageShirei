/// Get the length of a string.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// It's up to the caller to ensure that the pointer is valid.
///
/// # Arguments
///
/// * `string` - A pointer to a string.
///
/// # Returns
///
/// The length of the string.
pub unsafe fn string_length_a(string: *const u8) -> usize {
    let mut string2 = string;
    while !(*string2).is_null() {
        string2 = string2.add(1);
    }
    string2.offset_from(string) as usize
}

/// Get the length of a wide string.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
/// It's up to the caller to ensure that the pointer is valid.
///
/// # Arguments
///
/// * `string` - A pointer to a wide string.
///
/// # Returns
///
/// The length of the wide string.
pub unsafe fn string_length_w(string: *const u16) -> usize {
    let mut string2 = string;
    while !(*string2).is_null() {
        string2 = string2.add(1);
    }
    string2.offset_from(string) as usize
}

/// Trait for checking null terminator.
trait IsNull {
    /// Utility function for checking null terminator for u8 and u16
    fn is_null(&self) -> bool;
}

impl IsNull for u8 {
    fn is_null(&self) -> bool { *self == 0 }
}

impl IsNull for u16 {
    fn is_null(&self) -> bool { *self == 0 }
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

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

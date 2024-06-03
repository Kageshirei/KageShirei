use core::{ffi::c_void, ptr};

use crate::utils::string_length_w;

// use crate::utils::string_length_w;

// Definition of HANDLE and ULONG
pub type HANDLE = *mut c_void;
pub type ULONG = u32;
pub type PVOID = *mut c_void;

// Definition of LIST_ENTRY
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ListEntry {
    pub flink: *mut ListEntry,
    pub blink: *mut ListEntry,
}

// Definition of UNICODE_STRING
#[repr(C)]
pub struct UnicodeString {
    pub length: u16,
    pub maximum_length: u16,
    pub buffer: *mut u16,
}

impl UnicodeString {
    pub fn new() -> Self {
        UnicodeString {
            length: 0,
            maximum_length: 0,
            buffer: ptr::null_mut(),
        }
    }

    pub fn from_str(source_string: *const u16) -> Self {
        let mut unicode_string = UnicodeString::new();
        unicode_string.init(source_string);
        unicode_string
    }

    //RtlInitUnicodeString
    pub fn init(&mut self, source_string: *const u16) {
        if !source_string.is_null() {
            let dest_size = string_length_w(source_string) * 2; // 2 bytes per u16
            self.length = dest_size as u16;
            self.maximum_length = (dest_size + 2) as u16; // 2 bytes for the null terminator
            self.buffer = source_string as *mut u16;
        } else {
            self.length = 0;
            self.maximum_length = 0;
            self.buffer = ptr::null_mut();
        }
    }
}

#[repr(C)]
pub struct ObjectAttributes {
    pub length: ULONG,
    pub root_directory: HANDLE,
    pub object_name: *mut UnicodeString,
    pub attributes: ULONG,
    pub security_descriptor: PVOID,
    pub security_quality_of_service: PVOID,
}

impl ObjectAttributes {
    //InitializeObjectAttributes
    pub fn initialize(
        p: &mut ObjectAttributes,
        n: *mut UnicodeString,
        a: ULONG,
        r: HANDLE,
        s: PVOID,
    ) {
        p.length = core::mem::size_of::<ObjectAttributes>() as ULONG;
        p.root_directory = r;
        p.attributes = a;
        p.object_name = n;
        p.security_descriptor = s;
        p.security_quality_of_service = ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_string_length_w() {
    //     let string: [u16; 6] = [
    //         b'h' as u16,
    //         b'e' as u16,
    //         b'l' as u16,
    //         b'l' as u16,
    //         b'o' as u16,
    //         0,
    //     ];
    //     let length = string_length_w(string.as_ptr());
    //     assert_eq!(length, 5);
    // }

    #[test]
    fn test_rtl_init_unicode_string() {
        let source_string: [u16; 6] = [
            b'h' as u16,
            b'e' as u16,
            b'l' as u16,
            b'l' as u16,
            b'o' as u16,
            0,
        ];
        let mut destination_string = UnicodeString::new();

        destination_string.init(source_string.as_ptr());

        assert_eq!(destination_string.length, 5 * 2); // 5 characters, 2 bytes each
        assert_eq!(destination_string.maximum_length, (5 * 2 + 2) as u16); // 5 characters, 2 bytes each, plus 2 bytes for null terminator
        assert_eq!(
            destination_string.buffer,
            source_string.as_ptr() as *mut u16
        );
    }

    #[test]
    fn test_rtl_init_unicode_string_null_source() {
        let mut destination_string = UnicodeString::new();

        destination_string.init(ptr::null());

        assert_eq!(destination_string.length, 0);
        assert_eq!(destination_string.maximum_length, 0);
        assert_eq!(destination_string.buffer, ptr::null_mut());
    }

    #[test]
    fn test_initialize_object_attributes() {
        let mut object_name = UnicodeString {
            length: 0,
            maximum_length: 0,
            buffer: ptr::null_mut(),
        };

        let mut obj_attrs = ObjectAttributes {
            length: 0,
            root_directory: ptr::null_mut(),
            object_name: ptr::null_mut(),
            attributes: 0,
            security_descriptor: ptr::null_mut(),
            security_quality_of_service: ptr::null_mut(),
        };

        ObjectAttributes::initialize(
            &mut obj_attrs,
            &mut object_name,
            0x1234, // Some attributes
            ptr::null_mut(),
            ptr::null_mut(),
        );

        assert_eq!(
            obj_attrs.length,
            core::mem::size_of::<ObjectAttributes>() as ULONG
        );
        assert_eq!(obj_attrs.root_directory, ptr::null_mut());
        assert_eq!(obj_attrs.attributes, 0x1234);
        assert_eq!(obj_attrs.object_name, &mut object_name as *mut _);
        assert_eq!(obj_attrs.security_descriptor, ptr::null_mut());
        assert_eq!(obj_attrs.security_quality_of_service, ptr::null_mut());
    }
}

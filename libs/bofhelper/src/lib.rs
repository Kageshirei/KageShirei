#![no_std]

use core::ops::Deref;

pub use paste::paste;
use windows_sys::Win32::{
	Foundation::{BOOL, HANDLE},
	System::Threading::{PROCESS_INFORMATION, STARTUPINFOA},
};

/// Define a wrapper struct for BOF functions.
pub struct BOFFunctionWrapper<T>(T);

impl<T> Deref for BOFFunctionWrapper<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		// we have a pointer to a pointer that rust thinks is a pointer
		// deref is called when we call the function, so we use it to
		// dereference the pointer at runtime
		unsafe { &**(&self.0 as *const T as *const *const T) }
	}
}

impl<T> BOFFunctionWrapper<T> {
	/// Create a new BOFFunctionWrapper.
	pub const fn new(func: T) -> BOFFunctionWrapper<T> {
		BOFFunctionWrapper(func)
	}
}

/// this macro imports a symbol and then defines a wrapped version suitable for calling
#[macro_export]
macro_rules! import_function {
    ($pub:vis $lib:ident!$func:ident$args:tt $(-> $ret:ty)?) => {
        import_function!($pub $lib!$func$args $(-> $ret)?, "stdcall");
    };
    ($pub:vis $lib:ident!$func:ident$args:tt $(-> $ret:ty)?, $cc:literal) => {
        $crate::paste! {
            extern $cc {
                // hack: \x01 tells llvm not to add the _ on 32 bit
                // thanks alexchrichton: https://github.com/rust-lang/rust/issues/35052#issuecomment-235420755
                #[link_name = concat!("\x01__imp_", stringify!($lib), "$", stringify!($func))]
                fn [<__ $func>]$args $(-> $ret)?;
            }
            #[allow(non_upper_case_globals)]
            $pub const $func: $crate::BOFFunctionWrapper<unsafe extern $cc fn$args $(-> $ret)?> = $crate::BOFFunctionWrapper::new([<__ $func>]);
        }
    };
}

/// Macro to import internal functions using the "cdecl" calling convention.
macro_rules! import_internal_function {
    ($pub:vis $func:ident$args:tt $(-> $ret:ty)?) => {
        $crate::paste! {
            extern "cdecl" {
                // hack: \x01 tells llvm not to add the _ on 32 bit
                // thanks alexchrichton: https://github.com/rust-lang/rust/issues/35052#issuecomment-235420755
                #[link_name = concat!("\x01__imp_", stringify!($func))]
                fn [<__ $func>]$args $(-> $ret)?;
            }
            #[allow(non_upper_case_globals)]
            $pub const $func: $crate::BOFFunctionWrapper<unsafe extern "cdecl" fn$args $(-> $ret)?> = $crate::BOFFunctionWrapper::new([<__ $func>]);
        }
    }
}

#[cfg(target_arch = "x86")]
import_function!(NTDLL!_chkstk(), "cdecl");

/**
Manually disabled as it was causing a linker error:
i686-w64-mingw32-ld: ../../target/i686-pc-windows-gnu/release/deps/compiler_builtins-650c18b20ed02827.o:compiler_builtins.:(.text+0x88d0): multiple definition of `__chkstk'; ../../target/i686-pc-windows-gnu/release/deps/bofhelper-2b80530dde032518.o:bofhelper.9f47b150:(.text+0x0): first defined here

Re-enable it whenever you need it
 */
/*
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "C" fn __chkstk() {
    _chkstk()
}
*/

/// Define the _BOFData struct for parsing data.
#[repr(C)]
pub(crate) struct _BOFData {
	original: *mut u8,
	buffer: *mut u8,
	length: i32,
	size: i32,
}

/// Define the DataRelocation struct for handling data relocations.
#[repr(C, packed)]
pub struct DataRelocation {
	offset_to_sym: u32,
	offset_in_sec: u32,
	sec: u8,
	typ: u8,
}

// Define constants for relocation types and sections.
const IMAGE_REL_AMD64_ADDR64: u8 = 1;
const IMAGE_REL_I386_DIR32: u8 = 6;
const IMAGE_REL_AMD64_REL32: u8 = 4;
const REL_SEC_TEXT: u8 = 1;
const REL_SEC_DATA: u8 = 2;
const REL_SEC_RDATA: u8 = 3;


// hack: declare as function to prevent rust from including
// undefined refptr symbol
extern "C" {
	#[link_name = "\x01__data_start__"]
	fn __data_start__();
	#[link_name = "\x01__text_start__"]
	fn __text_start__();
	#[link_name = "\x01__rdata_start__"]
	fn __rdata_start__();
}

/// Perform relocations on the .data and .rdata sections  
/// # Safety  
/// I think you can guess why this is not safe at all  
pub unsafe fn bootstrap(relocs: &[u8]) -> Option<()> {
	let relocs = core::slice::from_raw_parts(
		relocs.as_ptr() as *const DataRelocation,
		relocs.len() / core::mem::size_of::<DataRelocation>(),
	);
	bootstrap_data(relocs, __rdata_start__ as usize)
}

/// Perform relocations on a single section
/// # Safety  
/// I think you can guess why this is not safe at all  
pub unsafe fn bootstrap_data(relocs: &[DataRelocation], section: usize) -> Option<()> {
	for reloc in relocs {
		let secbase = match reloc.sec {
			REL_SEC_TEXT => __text_start__ as usize,
			REL_SEC_DATA => __data_start__ as usize,
			REL_SEC_RDATA => __rdata_start__ as usize,
			_ => return None,
		};

		match reloc.typ {
			IMAGE_REL_AMD64_ADDR64 => {
				let ptr: *mut u64 = (section + reloc.offset_in_sec as usize) as *mut u64;
				*ptr += (secbase + reloc.offset_to_sym as usize) as u64;
			}
			IMAGE_REL_I386_DIR32 => {
				let ptr: *mut u32 =
					(section as *mut u8).add(reloc.offset_in_sec as usize) as *mut u32;
				*ptr += (secbase + reloc.offset_to_sym as usize) as u32;
			}
			IMAGE_REL_AMD64_REL32 => {
				// relative to the byte following the relocation
				// so find the distance between the symbol and the current_loc + 4
				let ptr: *mut u32 = (section as *mut u8).add(reloc.offset_in_sec as usize) as *mut u32;
				*ptr = (secbase - (ptr.add(1) as usize) + reloc.offset_to_sym as usize) as u32;
			}
			_ => return None,
		}
	}

	Some(())
}

// Import various internal functions for Beacon data manipulation.
import_internal_function!(BeaconDataParse(
    parser: *mut _BOFData,
    buffer: *mut u8,
    size: i32
));
import_internal_function!(BeaconDataPtr(parser: *mut _BOFData, size: i32) -> *mut u8);
import_internal_function!(BeaconDataInt(parser: *mut _BOFData) -> i32);
import_internal_function!(BeaconDataShort(parser: *mut _BOFData) -> i16);
import_internal_function!(BeaconDataLength(parser: *const _BOFData) -> i32);
import_internal_function!(BeaconDataExtract(parser: *mut _BOFData, size: *mut i32) -> *mut u8);

/// Define the BofData struct to manage Beacon data parsing.
pub struct BofData(_BOFData);

impl BofData {
	/// Parse data into BofData
	pub fn parse(buffer: *mut u8, size: i32) -> Self {
		let mut parser = unsafe { core::mem::zeroed() };
		unsafe { BeaconDataParse(&mut parser, buffer, size) };
		Self(parser)
	}

	/// Get a pointer to data of specified size.
	pub fn get_ptr(&mut self, size: i32) -> *mut u8 {
		unsafe { BeaconDataPtr(&mut self.0, size) }
	}

	/// Get an integer from the data.
	pub fn get_int(&mut self) -> i32 {
		unsafe { BeaconDataInt(&mut self.0) }
	}

	/// Get a short integer from the data.
	pub fn get_short(&mut self) -> i16 {
		unsafe { BeaconDataShort(&mut self.0) }
	}

	/// Get the length of the data.
	pub fn len(&self) -> i32 {
		unsafe { BeaconDataLength(&self.0) }
	}

	/// Check if the data is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Get the data as a byte slice.
	pub fn get_data<'a>(&mut self) -> &'a [u8] {
		let mut len = 0i32;
		let ret = unsafe { BeaconDataExtract(&mut self.0, &mut len) };
		unsafe { core::slice::from_raw_parts_mut(ret, len as usize) }
	}

	/// Get the data as a string.
	pub fn get_str<'a>(&mut self) -> &'a str {
		let data = self.get_data();
		// strip off the null
		unsafe { core::str::from_utf8_unchecked(&data[..data.len() - 1]) }
	}
}

/// Define the BOFFormat struct for formatted data.
#[repr(C)]
pub struct BOFFormat {
	original: *mut u8,
	buffer: *mut u8,
	length: i32,
	size: i32,
}

// Define constants for callback types.
pub const CALLBACK_OUTPUT: i32 = 0;
pub const CALLBACK_OUTPUT_OEM: i32 = 0x1e;
pub const CALLBACK_OUTPUT_UTF8: i32 = 0x20;
pub const CALLBACK_ERROR: i32 = 0xd;

// Import various internal functions for Beacon interactions.
import_internal_function!(pub BeaconOutput(typ: i32, data: *const u8, len: i32));
import_internal_function!(pub BeaconPrintf(typ: i32, fmt: *const u8, ...));

/// Macro to print a message using BeaconPrintf.
#[macro_export]
macro_rules! beacon_print {
    ($($args:tt)*) => {
        {
            let mut s = format!($($args)*);
            s.push('\0');
            #[allow(unused_unsafe)]
            unsafe { $crate::BeaconPrintf(CALLBACK_OUTPUT, s.as_ptr()) };
        }
    };
}

/// Macro to print an error message using BeaconPrintf.
#[macro_export]
macro_rules! beacon_print_error {
    ($($args:tt)*) => {
        {
            let mut s = format!($($args)*);
            s.push('\0');
            #[allow(unused_unsafe)]
            unsafe { $crate::BeaconPrintf(CALLBACK_ERROR, s.as_ptr()) };
        }
    };
}

// Import additional internal functions for token manipulation and process injection.
// the BeaconFormat* apis were not included here because we
// can just use rust's formatters
import_internal_function!(pub BeaconUseToken(token: HANDLE) -> BOOL);
import_internal_function!(pub BeaconRevertToken());
import_internal_function!(pub BeaconIsAdmin());
import_internal_function!(pub BeaconGetSpawnTo(x86: BOOL, buffer: *mut u8, length: i32));
import_internal_function!(pub BeaconInjectProcess(
    hProc: HANDLE,
    pid: i32,
    payload: *mut u8,
    p_len: i32,
    p_offset: i32,
    arg: *mut u8,
    a_len: i32
));
import_internal_function!(pub BeaconInjectTemporaryProcess(
    pInfo: *mut PROCESS_INFORMATION,
    payload: *mut u8,
    p_len: i32,
    p_offset: i32,
    arg: *mut u8,
    a_len: i32
));
import_internal_function!(pub BeaconSpawnTemporaryProcess(x86: BOOL, ignoreToken: BOOL, si: *mut STARTUPINFOA, pInfo: *mut PROCESS_INFORMATION) -> BOOL);
import_internal_function!(pub BeaconCleanupProcess(pInfo: *mut PROCESS_INFORMATION));
import_internal_function!(pub toWideChar(src: *mut u8, dst: *mut u16, max: i32) -> BOOL);

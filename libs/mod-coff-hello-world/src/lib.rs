#![no_std]

extern crate alloc;
extern crate wee_alloc;

use alloc::string::String;
use core::arch::asm;
use core::ffi::{c_char, c_int};

use bofentry::bof;
// use bofhelper::{beacon_print, BofData, CALLBACK_OUTPUT, import_function};
use bofhelper::BofData;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern "C" {
	// Declare an external C function to avoid creating additional symbols.
	fn BeaconOutput(_type: c_int, data: *mut c_char, len: c_int);
}

/// is generic output. Cobalt Strike will convert this output to UTF-16 (internally) using the target's default character set.
#[allow(dead_code)]
const CALLBACK_OUTPUT: u32 = 0x0;
/// is generic output. Cobalt Strike will convert this output to UTF-16 (internally) using the target's OEM character set. You probably won't need this, unless you're dealing with output from cmd.exe.
#[allow(dead_code)]
const CALLBACK_OUTPUT_OEM: u32 = 0x1e;
/// is a generic error message.
#[allow(dead_code)]
const CALLBACK_OUTPUT_UTF8: u32 = 0x20;
/// is generic output. Cobalt Strike will convert this output to UTF-16 (internally) from UTF-8.
#[allow(dead_code)]
const CALLBACK_ERROR: u32 = 0x0d;


// you can specify the export name in the proc macro or just use it bare
// to have it use the function name!
#[bof(go)]
fn entrypoint(mut data: BofData) {
	unsafe {
		asm! {
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", "nop", "nop", "nop", "nop",
		"nop", // Trigger a breakpoint
		options(nostack, nomem),
		}
	}
	unsafe {
		let mut str = String::from("Hello, World!\0");
		BeaconOutput(CALLBACK_OUTPUT as i32, str.as_mut_ptr() as *mut i8, 14);
	}
	unsafe {
		asm! {
		"nop",
		"nop",
		"nop",
		"nop",
		"nop",
		"nop",
		"nop",
		options(nostack, nomem),
		}
	}
}
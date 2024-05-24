use std::ffi::{c_char, c_int};

use bofentry::bof;
use bofhelper::BofData;

extern "C" {
	fn BeaconOutput(_type: c_int, data: *mut c_char, len: c_int);
}

// you can specify the export name in the proc macro or just use it bare
// to have it use the function name!
#[bof(go)]
fn entrypoint(mut data: &BofData) {
	let mut str = Box::new("test from bof\0".to_string());
	unsafe {
		BeaconOutput(0x0, str.as_mut_ptr() as *mut c_char, str.len() as c_int);
	}
}
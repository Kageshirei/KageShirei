#![no_std]
#![allow(internal_features)]
#![feature(core_intrinsics, lang_items)]
#![no_main]

extern crate alloc;// extern crate libc;
extern crate wee_alloc;

#[cfg(feature = "wasmer")]
mod wasmer_imp;
#[cfg(feature = "wasmtime")]
mod wasmtime_imp;
#[cfg(feature = "wasmi")]
mod wasmi_imp;
#[cfg(feature = "wain")]
mod wain_imp;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
	core::intrinsics::abort();
}

#[no_mangle]
extern "C" fn _start() {
	#[cfg(feature = "wasmer")]
	wasmer_imp::run();

	#[cfg(feature = "wasmtime")]
	wasmtime_imp::run();

	#[cfg(feature = "wasmi")]
	wasmi_imp::run();

	#[cfg(feature = "wain")]
	wain_imp::run();
}


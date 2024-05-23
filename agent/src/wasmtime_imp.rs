/*
 * This implementation requires a previous pass with a compile function with one of the compiler available in wasmtime (cranelift).
 * The size in a no_std environment is 1.4MB, which is too big for the current implementation.
 */

use libc::size_t;
use wasmtime::{Caller, Config, Engine, Linker, Memory, Module, Store};

// static WASM: &'static [u8] = include_bytes!("/home/ebalo/Desktop/Projects/rust/rs2/target/wasm32-unknown-unknown/release/mod-wasm-hello-world.wasm");
static WASM_SERIALIZED: &'static [u8] = include_bytes!("/home/ebalo/Desktop/Projects/rust/rs2/mod-wasm-hello-world.wasmtime");

pub fn run() {
	let mut config = Config::new();
	unsafe {
		config.detect_host_feature(|function| {
			match function {
				"sse3" => Some(true),
				"ssse3" => Some(true),
				"sse4.1" => Some(true),
				"sse4.2" => Some(true),
				"popcnt" => Some(true),
				"avx" => Some(true),
				"avx2" => Some(true),
				"fma" => Some(true),
				"bmi1" => Some(true),
				"bmi2" => Some(true),
				"avx512bitalg" => Some(true),
				"avx512dq" => Some(true),
				"avx512f" => Some(true),
				"avx512vl" => Some(true),
				"avx512vbmi" => Some(true),
				"lzcnt" => Some(true),
				_ => Some(false),
			}
		});
	}

	let engine = Engine::new(&config).unwrap();
	let mut store: Store<Option<Memory>> = Store::new(&engine, None);

	// Modules can be compiled through either the text or binary format
	// let module = Module::new(store.engine(), WASM).unwrap();
	let module = unsafe { Module::deserialize(store.engine(), WASM_SERIALIZED) }.unwrap();
	// let serialized = module.serialize().unwrap();
	// std::fs::write("mod-wasm-hello-world.wasmtime", &serialized).unwrap();

	// Host functionality can be arbitrary Rust functions and is provided
	// to guests through a `Linker`.
	let mut linker = Linker::new(&engine);
	linker.func_wrap("env", "print_str", |caller: Caller<'_, Option<Memory>>, ptr: u32, len: u32| {
		let memory = caller.data().as_ref().unwrap();
		let mem_data = memory.data(&caller);
		let text: &[u8] = &mem_data[ptr as usize..(ptr + len) as usize];

		unsafe {
			libc::write(libc::STDOUT_FILENO, text.as_ptr() as *const _, len as size_t);
		}
	}).unwrap();

	// Instantiation of a module requires specifying its imports and then
	// afterwards we can fetch exports by name, as well as asserting the
	// type signature of the function with `get_typed_func`.
	let instance = linker.instantiate(&mut store, &module).unwrap();
	let memory = instance.get_memory(&mut store, "memory").unwrap();
	store.data_mut().replace(memory);

	let hello = instance.get_typed_func::<(), ()>(&mut store, "hello_wasm").unwrap();

	// And finally we can call the wasm!
	hello.call(&mut store, ()).unwrap();
}
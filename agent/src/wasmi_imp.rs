/*
 * The size in a no_std environment is 1.7MB, which is too big for the current implementation.
 */

use libc::size_t;
use wasmi::{Caller, Engine, Linker, Memory, Module, Store};

static WASM: &'static [u8] = include_bytes!("/home/ebalo/Desktop/Projects/rust/rs2/target/wasm32-unknown-unknown/release/mod-wasm-hello-world.wasm");

pub fn run() {
	let engine = Engine::default();
	let mut store: Store<Option<Memory>> = Store::new(&engine, None);

	// Modules can be compiled through either the text or binary format
	let module = Module::new(store.engine(), WASM).unwrap();

	// Host functionality can be arbitrary Rust functions and is provided
	// to guests through a `Linker`.
	let mut linker = Linker::new(&engine);
	linker.func_wrap("env", "print_str", |caller: Caller<'_, Option<Memory>>, ptr: u32, len: u32| {
		let memory = caller.data().as_ref().unwrap();
		let mem_data = memory.data(&caller);
		let mut mut_ptr = ptr as u64;
		let text: &[u8] = &mem_data[ptr as usize..(ptr + len) as usize];

		unsafe {
			libc::write(libc::STDOUT_FILENO, text.as_ptr() as *const _, len as size_t);
		}
	}).unwrap();

	// Instantiation of a module requires specifying its imports and then
	// afterwards we can fetch exports by name, as well as asserting the
	// type signature of the function with `get_typed_func`.
	let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
	let memory = instance.get_memory(&mut store, "memory").unwrap();
	store.data_mut().replace(memory);

	let hello = instance.get_typed_func::<(), ()>(&mut store, "hello_wasm").unwrap();

	// And finally we can call the wasm!
	hello.call(&mut store, ()).unwrap();
}
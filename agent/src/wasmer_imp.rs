/*
 * This implementation requires a previous pass with a compile function with one of the compiler available in wasmer (llvm more optimized, or cranelift).
 * The size in a no_std environment is 1.7MB, which is too big for the current implementation.
 */

use alloc::vec::Vec;

use libc::size_t;
use wasmer::{AsStoreRef, Function, FunctionEnv, FunctionEnvMut, imports, Instance, Memory, MemoryAccessError, MemoryView, Module, Store, WasmPtr};

pub unsafe fn read_string(view: &MemoryView, start: u32, len: u32) -> Result<Vec<u8>, MemoryAccessError> {
	let ptr = WasmPtr::<u8>::new(start);
	let mut read_len = 0u32;
	let mut data = ptr.read_until(view, |v| {
		if read_len < len {
			read_len += 1;
			return false;
		}
		true
	}).unwrap();
	Ok(data.clone())
}

struct Env {
	memory: Option<Memory>,
}

impl Env {
	fn set_memory(&mut self, memory: Memory) {
		self.memory = Some(memory);
	}

	fn get_memory(&self) -> &Memory {
		self.memory.as_ref().unwrap()
	}

	fn view<'a>(&'a self, store: &'a impl AsStoreRef) -> MemoryView<'a> {
		self.get_memory().view(store)
	}
}

pub fn run() {
	let mut store = Store::default();

	// let memory = Memory::new(&mut store, MemoryType::new(1, None, true)).unwrap();

	let env = FunctionEnv::new(&mut store, Env { memory: None });

	// Let's define the import object used to import our function
	// into our webassembly sample application.
	//
	// We've defined a macro that makes it super easy.
	//
	// The signature tells the runtime what the signature (the parameter
	// and return types) of the function we're defining here is.
	// The allowed types are `i32`, `u32`, `i64`, `u64`,
	// `f32`, and `f64`.
	//
	// Make sure to check this carefully!
	let import_object = imports! {
        // Define the "env" namespace that was implicitly used
        // by our sample application.
        "env" => {
            "print_str" => Function::new_typed_with_env(&mut store, &env, print_str),
            // "memory" => memory,
        },
    };


	// let module = Module::new(&store, WASM).unwrap();
	// module.serialize_to_file("sample.serialized").unwrap();
	let module = unsafe { Module::deserialize_from_file(&store, "/home/ebalo/Desktop/Projects/rust/rs2/sample.serialized") }.unwrap();

	// Compile our webassembly into an `Instance`.
	let instance = Instance::new(&mut store, &module, &import_object).unwrap();
	let instance_memory = instance.exports.get_memory("memory").unwrap();

	env.as_mut(&mut store).set_memory(instance_memory.clone());

	// Call our exported function!
	instance.exports.get_function("hello_wasm").unwrap().call(&mut store, &[]).unwrap();
}

// Let's define our "print_str" function.
//
// The declaration must start with "extern" or "extern "C"".
fn print_str(env: FunctionEnvMut<Env>, ptr: u32, len: u32) {
	let view = env.data().view(&env);
	let text = unsafe { read_string(&view, ptr, len).unwrap() };

	unsafe {
		libc::write(libc::STDOUT_FILENO, text.as_ptr() as *const _, len as size_t);
	}
	// Print it!
	// println!("{}", text);
}
use alloc::string::ToString;

static WASM: &'static [u8] = include_bytes!("/home/ebalo/Desktop/Projects/rust/rs2/target/wasm32-unknown-unknown/release/mod-wasm-hello-world.wasm");

struct WainImporter;

impl wain_exec::Importer for WainImporter {
	fn validate(&self, name: &str, params: &[wain_ast::ValType], ret: Option<wain_ast::ValType>) -> Option<wain_exec::ImportInvalidError> {
		match name {
			"print_str" => {
				if params.len() == 2 && ret.is_none() && params[0] == wain_ast::ValType::I32 && params[1] == wain_ast::ValType::I32 {
					None
				} else {
					Some(wain_exec::ImportInvalidError::SignatureMismatch { expected_params: &[wain_ast::ValType::I32, wain_ast::ValType::I32], expected_ret: None })
				}
			}
			_ => Some(wain_exec::ImportInvalidError::NotFound),
		}
		// `name` is a name of function to validate. `params` and `ret` are the function's signature.
		// Return ImportInvalidError::NotFound when the name is unknown.
		// Return ImportInvalidError::SignatureMismatch when signature does not match.
		// wain_exec::check_func_signature() utility is would be useful for the check.
	}
	fn call(&mut self, name: &str, stack: &mut wain_exec::Stack, memory: &mut wain_exec::Memory) -> Result<(), wain_exec::ImportInvokeError> {
		match name {
			"print_str" => {
				let len = stack.pop::<i32>() as u32;
				let ptr = stack.pop::<i32>() as u32;
				let mem_data = memory.data();
				let text: &[u8] = &mem_data[ptr as usize..(ptr + len) as usize];
				unsafe {
					libc::write(libc::STDOUT_FILENO, text.as_ptr() as *const _, len as usize);
				}
				Ok(())
			}
			_ => Err(wain_exec::ImportInvokeError::Fatal { message: "unknown function".to_string() }),
		}
		// Implement your own function call. `name` is a name of function and you have full access
		// to stack and linear memory. Pop values from stack for getting arguments and push value to
		// set return value.
		// Note: Consistency between imported function signature and implementation of this method
		// is your responsibility.
		// On invocation failure, return ImportInvokeError::Fatal. It is trapped by interpreter and it
		// stops execution immediately.
	}
}

pub fn run() {
	let tree = match wain_syntax_binary::parse(WASM) {
		Ok(tree) => tree,
		Err(e) => {
			return;
		}
	};
	let mut runtime = match wain_exec::Runtime::instantiate(&tree.module, WainImporter {}) {
		Ok(runtime) => runtime,
		Err(e) => {
			return;
		}
	};

	match runtime.invoke("hello_wasm", &[]) {
		Ok(_) => {}
		Err(e) => {
			return;
		}
	};
}
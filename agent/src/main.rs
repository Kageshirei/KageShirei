// mod wasmtime_imp;

use coffee_ldr::loader::Coffee;

fn main() {
	// wasmtime_imp::run();
	Coffee::new().run();
}


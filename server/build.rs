fn main() {
	// rerun migration import if the migrations folder changes
	println!("cargo:rerun-if-changed=./migrations");
}
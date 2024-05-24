// Import the necessary type for defining a procedural macro.
use proc_macro::TokenStream;

/// This attribute macro transforms a function into an unsafe external C function with BOF (Beacon Object File) specific logic.
///
/// # Arguments
/// * `args` - Additional arguments for the macro (used for the exported function name).
/// * `input` - The input TokenStream representing the original function.
#[proc_macro_attribute]
pub fn bof(args: TokenStream, input: TokenStream) -> TokenStream {
	// Parse the input TokenStream into a syn::ItemFn, representing the function.
	let syn::ItemFn { attrs, vis, sig, block } = syn::parse_macro_input!(input);

	// Extract the function's identifier (name).
	let fn_ident = &sig.ident;

	// Extract the statements within the function's block.
	let stmts = &block.stmts;

	// Determine the identifier for the exported function.
	// If no arguments are provided, use the original function's name.
	// Otherwise, parse the provided arguments to get the export identifier.
	let export_ident: syn::Ident = if args.is_empty() {
		fn_ident.clone()
	} else {
		syn::parse_macro_input!(args)
	};

	// Generate the output TokenStream, defining the transformed function.
	quote::quote! {
        // Mark the function to avoid name mangling.
        #[no_mangle]
        // Define the function as unsafe and compatible with C calling conventions.
        unsafe extern "C" fn #export_ident(args: *mut u8, alen: i32) {
            // Parse the BOF data from the arguments.
            let mut data = bofhelper::BofData::parse(args, alen);

            // Perform BOF relocation bootstrap.
            // If it fails, print an error message and return.
            /*if bofhelper::bootstrap(data.get_data()).is_none() {
                bofhelper::BeaconPrintf(
                    bofhelper::CALLBACK_ERROR,
                    "BOF relocation bootstrap failed\0".as_ptr(),
                );
                return;
            }*/

            // Initialize the allocator if the "alloc" feature is enabled.
            // #[cfg(feature = "alloc")]
            // bofalloc::ALLOCATOR.initialize();

            // Inject the original function's attributes, visibility, signature, and statements.
            #(#attrs)* #vis #sig {
                #(#stmts)*
            }

            // Call the original function with the parsed BOF data.
            #fn_ident(&mut data);

            // Destroy the allocator if the "alloc" feature is enabled.
            // #[cfg(feature = "alloc")]
            // bofalloc::ALLOCATOR.destroy();
        }
    }.into() // Convert the generated code back into a TokenStream.
}

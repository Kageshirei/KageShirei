//! # Kageshirei Extensions Macros
//! This crate provides a set of custom procedural macros for use in the Kageshirei project and its
//! extensions.

#![allow(
    clippy::panic,
    reason = "The macro is used for code generation and panics are used for error handling, it is expected a macro \
              fails to compile if something goes wrong"
)]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "Simple matches have been used for simplicity, adding all cases would make the code more complex and \
              difficult to read"
)]
#![allow(
    clippy::pattern_type_mismatch,
    reason = "Ok as in this case simplifies the code"
)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, GenericArgument, ItemFn, Pat, PatType, PathArguments, Type};

/// A helper function to debug types at runtime (not used in this macro but can be handy for
/// development).
#[allow(
    dead_code,
    reason = "Not used in the macro but can be handy for development"
)]
fn print_type<T>(_: T) -> String { std::any::type_name::<T>().to_owned() }

/// Attribute macro for generating hook registration functionality.
///
/// This macro takes an async function as input and generates a struct with two methods:
/// - `register`: Registers the function as a hook, returning a closure.
/// - `call`: Calls the function directly with the provided context.
///
/// The input function must have the following characteristics:
/// 1. It must be asynchronous.
/// 2. Its first argument must be of type `Arc<Box<T>>`, where `T` is the context type.
/// 3. It must return `Result<(), String>`.
///
/// Example usage:
/// ```rust ignore
/// #[registerable_hook]
/// async fn my_hook(context: Arc<Box<MyContext>>) -> Result<(), String> {
///     // Logic here
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn registerable_hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function into an `ItemFn` AST node.
    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract the function name (identifier).
    let fn_name = &input_fn.sig.ident;

    // Extract the type of the first argument in the function signature.
    let first_arg_type = &input_fn.sig.inputs.first().unwrap();

    // Analyze the first argument to extract the context type (i.e., the type inside `Arc<Box<T>>`).
    let context_type = match first_arg_type {
        // Ensure the first argument is a typed parameter.
        FnArg::Typed(PatType {
            ty,
            ..
        }) => {
            match &**ty {
                Type::Path(path) => {
                    // Extract the type path segments to verify it's `Arc<Box<T>>`.
                    let segments = &path.path.segments;
                    let first_segment = segments.last().unwrap();
                    let first_segment_name = first_segment.ident.to_string();

                    // Check if the first segment is `Arc`.
                    if first_segment_name == "Arc" {
                        // Extract the generic arguments of `Arc`.
                        let PathArguments::AngleBracketed(args) = &first_segment.arguments
                        else {
                            panic!("Expected type `Arc<T>` but found something else.");
                        };

                        // Get the inner type of `Arc<T>`.
                        let GenericArgument::Type(inner_type) = args.args.first().unwrap()
                        else {
                            panic!("Expected a generic type argument, found something else.");
                        };

                        // Ensure the inner type is a path type (e.g., `Box<T>`).
                        let Type::Path(inner_path) = inner_type
                        else {
                            panic!("Expected a path type argument, found something else.");
                        };

                        // Extract the second segment (e.g., `Box`).
                        let second_segment = inner_path.path.segments.last().unwrap();
                        let second_segment_name = second_segment.ident.to_string();

                        // Check if the second segment is `Box`.
                        if second_segment_name == "Box" {
                            // Extract the generic arguments of `Box<T>`.
                            let PathArguments::AngleBracketed(box_args) = &second_segment.arguments
                            else {
                                panic!("Expected type `Arc<Box<T>>` but found something else.");
                            };

                            // Get the inner type of `Box<T>` (i.e., `T`).
                            let GenericArgument::Type(context_type) = box_args.args.first().unwrap()
                            else {
                                panic!("Expected a generic type argument, found something else.");
                            };

                            // Return the extracted context type.
                            quote! { #context_type }
                        }
                        else {
                            panic!("Expected type `Arc<Box<T>>` but found something else.");
                        }
                    }
                    else {
                        panic!("Expected type `Arc<Box<T>>` but found something else.");
                    }
                },
                _ => panic!("Expected a reference to a type like `Arc<Box<T>>`."),
            }
        },
        _ => panic!("Expected the function's first argument to be a typed parameter."),
    };

    // Extract the argument name (identifier) of the first parameter.
    let FnArg::Typed(context_arg) = input_fn.sig.inputs.first().unwrap()
    else {
        panic!("Function signature requires a parameter to handle the context")
    };
    let Pat::Ident(ref context_arg_name) = *context_arg.pat
    else {
        panic!("Cannot extract the context argument name")
    };

    // Extract the body of the input function.
    let fn_body = input_fn.block.stmts;

    // Generate the struct and its implementation
    let expanded = quote! {
        /// Automatically generated struct to represent the hook.
        pub struct #fn_name;

        impl #fn_name {
            /// Register the function as a hook.
            ///
            /// Returns a closure that takes a reference to the context (`Arc<Box<T>>`) and
            /// asynchronously invokes the hook logic via the `call` method.
            ///
            /// # Returns
            /// A closure that wraps the hook function call.
            pub fn register() -> impl Fn(&Arc<Box<#context_type>>) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
                move |ctx: &Arc<Box<#context_type>>| {
                    Box::pin(Self::call(ctx.clone()))
                }
            }

            /// Invoke the hook logic directly.
            ///
            /// This method provides the primary hook logic and is called either directly
            /// or indirectly via the closure returned by `register`.
            ///
            /// # Arguments
            ///
            /// * `#context_arg_name` - The context argument of type `Arc<Box<#context_type>>`.
            ///
            /// # Returns
            ///
            /// An `async` result indicating success (`Ok(())`) or failure (`Err(String)`).
            pub async fn call(#context_arg_name: Arc<Box<#context_type>>) -> Result<(), String> {
                // Original function logic
                #(#fn_body)*
            }
        }
    };

    // Convert the generated code into a TokenStream to be returned by the macro.
    TokenStream::from(expanded)
}

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(FieldNames)]
pub fn field_names_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;

	// Ensure the input is a struct with named fields
	let fields = if let syn::Data::Struct(data_struct) = input.data {
		if let syn::Fields::Named(fields_named) = data_struct.fields {
			fields_named.named
		} else {
			panic!("FieldNames can only be derived for structs with named fields");
		}
	} else {
		panic!("FieldNames can only be derived for structs");
	};

	let field_variants: Vec<_> = fields.iter().map(|f| {
		let ident = f.ident.as_ref().unwrap();
		quote! { #ident }
	}).collect();

	let enum_name = syn::Ident::new(&format!("{}Fields", name), name.span());

	let expanded = quote! {
        pub enum #enum_name {
            #(#field_variants),*
        }
    };

	TokenStream::from(expanded)
}
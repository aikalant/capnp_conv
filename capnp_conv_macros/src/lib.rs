#![deny(
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::string_slice,
    clippy::pedantic
)]
#![forbid(unsafe_code)]

mod generators;
mod models;
mod parsers;
mod utils;

use models::ItemInfo;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};
use utils::is_capnp_attr;

#[proc_macro_attribute]
pub fn capnp_conv(attr_stream: TokenStream, input_stream: TokenStream) -> TokenStream {
    let capnp_struct = parse_macro_input!(attr_stream as Path);
    let mut input = parse_macro_input!(input_stream as DeriveInput);

    let output = match ItemInfo::parse_input(&input) {
        Ok(item_info) => {
            let impls = item_info.generate_impls(&capnp_struct);
            remove_capnp_field_attrs(&mut input);
            quote! {
                #input
                #impls
            }
        }
        Err(error) => error.to_compile_error(),
    };

    output.into()
}

fn remove_capnp_field_attrs(input: &mut DeriveInput) {
    match &mut input.data {
        syn::Data::Struct(data) => {
            for field in &mut data.fields {
                field.attrs.retain(|attr| !is_capnp_attr(attr));
            }
        }
        syn::Data::Enum(data) => {
            for variant in &mut data.variants {
                variant.attrs.retain(|attr| !is_capnp_attr(attr));
            }
        }
        syn::Data::Union(_) => unimplemented!(),
    }
}

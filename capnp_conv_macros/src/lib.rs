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

    match ItemInfo::parse_input(&input) {
        Ok(item_info) => {
            let output = item_info.generate_impls(&capnp_struct);
            remove_capnp_field_attrs(&mut input);
            quote! {
              #input
              #output
            }
        }
        Err(error) => error.to_compile_error(),
    }
    .into()
}

fn remove_capnp_field_attrs(input: &mut DeriveInput) {
    match &mut input.data {
        syn::Data::Struct(data) => {
            for field in &mut data.fields {
                drain_filter(&mut field.attrs, is_capnp_attr);
            }
        }
        syn::Data::Enum(data) => {
            for variant in &mut data.variants {
                drain_filter(&mut variant.attrs, is_capnp_attr);
            }
        }
        syn::Data::Union(_) => unimplemented!(),
    }
}

//not using nightly so we need to do this manually
fn drain_filter<T>(vec: &mut Vec<T>, predicate: fn(&T) -> bool) {
    let mut i = 0;
    while i != vec.len() {
        if predicate(&vec[i]) {
            vec.remove(i);
        } else {
            i += 1;
        }
    }
}

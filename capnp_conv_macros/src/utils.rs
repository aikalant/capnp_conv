use std::fmt::Display;

use proc_macro2::{Ident, Span};
use quote::{format_ident, IdentFragment};
use syn::{
    token::Colon2, AttrStyle, Attribute, Error, GenericArgument, Path, PathArguments, Result, Type,
};

use crate::models::FieldType;

pub fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, format!("capnp_conv: {}", message)))
}

pub fn is_capnp_attr(attribute: &&Attribute) -> bool {
    attribute.style == AttrStyle::Outer
        && attribute.path.segments.last().unwrap().ident == "capnp_conv"
}

pub fn to_ident(fragment: impl IdentFragment) -> Ident {
    format_ident!("{}", fragment)
}

pub fn to_capnp_generic(generic: &Ident) -> Ident {
    format_ident!("__CaPnP__{}", generic)
}

/// for a type of `Option::<bool>`, will return `"Option"`, the `bool` subtype
pub fn try_peel_type(ty: &Type) -> Option<(&Ident, &Type)> {
    if let Type::Path(type_path) = ty {
        let last_segment = type_path.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(arguments) = &last_segment.arguments {
            if arguments.args.len() == 1 {
                if let GenericArgument::Type(sub_type) = arguments.args.first().unwrap() {
                    return Some((&last_segment.ident, sub_type));
                }
            }
        }
    }
    None
}

/// Turns `Foo<T, Bar<Y>>` into `Foo::<T, Bar::<Y>>`
pub fn as_turbofish(path: &Path) -> Path {
    let mut path = path.clone();
    for segment in &mut path.segments {
        if let PathArguments::AngleBracketed(bracketed) = &mut segment.arguments {
            bracketed.colon2_token = Some(Colon2::default())
        }
    }
    path
}

pub fn is_ptr_type(field_type: &FieldType) -> bool {
    matches!(
        field_type,
        FieldType::Blob(_)
            | FieldType::Struct(_)
            | FieldType::List(_)
            | FieldType::GenericStruct(_, _)
    )
}

// copied from how https://github.com/capnproto/capnproto-rust/blob/master/capnpc
// generates enum names
pub fn capitalize_first_letter(s: &str) -> String {
    let mut result_chars: Vec<char> = Vec::new();
    for c in s.chars() {
        result_chars.push(c)
    }
    result_chars[0] = result_chars[0].to_ascii_uppercase();
    result_chars.into_iter().collect()
}

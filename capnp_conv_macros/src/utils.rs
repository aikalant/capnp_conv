use std::fmt::Display;

use proc_macro2::{Ident, Span};
use quote::{format_ident, IdentFragment};
use syn::{
    AttrStyle, Attribute, Error, GenericArgument, Path, PathArguments, Result, Type,
    token::PathSep,
};

use crate::models::FieldType;

pub fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, format!("capnp_conv: {message}")))
}

pub fn is_capnp_attr(attribute: &Attribute) -> bool {
    attribute.style == AttrStyle::Outer
        && attribute.path().is_ident("capnp_conv")
}

pub fn to_ident(fragment: impl IdentFragment) -> Ident {
    format_ident!("{}", fragment)
}

pub fn to_capnp_generic(generic: &Ident) -> Ident {
    format_ident!("__CaPnP__{}", generic)
}

/// For a type like `Option<bool>`, returns `Some(("Option", &bool))`.
/// Returns `None` if the type is not a path, has no segments,
/// the last segment has no angle-bracketed arguments, or has zero or more than one generic type argument.
pub fn try_peel_type(ty: &Type) -> Option<(&Ident, &Type)> {
    let type_path = if let Type::Path(type_path) = ty {
        type_path
    } else {
        return None;
    };

    let last_segment = type_path.path.segments.last()?;

    let arguments = if let PathArguments::AngleBracketed(arguments) = &last_segment.arguments {
        arguments
    } else {
        return None;
    };

    // Ensure exactly one generic argument which is a type
    if arguments.args.len() == 1 {
         if let Some(GenericArgument::Type(sub_type)) = arguments.args.first() {
             return Some((&last_segment.ident, sub_type));
         }
    }

    None
}


/// Turns `Foo<T, Bar<Y>>` into `Foo::<T, Bar::<Y>>` by adding `::` before generic arguments.
pub fn as_turbofish(path: &Path) -> Path {
    let mut path = path.clone();
    for segment in &mut path.segments {
        if let PathArguments::AngleBracketed(bracketed) = &mut segment.arguments {
            bracketed.colon2_token = Some(PathSep::default());
        }
    }
    path
}

/// Checks if the field type corresponds to a Cap'n Proto pointer type.
pub const fn is_ptr_type(field_type: &FieldType) -> bool {
    matches!(
        field_type,
        FieldType::Data(_)
            | FieldType::Text(_)
            | FieldType::Struct(_)
            | FieldType::List(_)
            | FieldType::GenericStruct(_)
    )
}

/// Capitalizes the first ASCII character of a string.
/// Returns an empty string if the input string is empty.
pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            // Pre-allocate string capacity for efficiency.
            let mut result = String::with_capacity(s.len());
            result.push(first.to_ascii_uppercase());
            // Append the rest of the string slice efficiently.
            result.push_str(chars.as_str());
            result
        }
    }
}

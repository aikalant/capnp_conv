use std::{
    collections::{HashMap, hash_map},
    mem::discriminant,
};

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, GenericArgument,
    GenericParam, Generics, LitStr, Path, PathArguments, Result, Type, Variant, spanned::Spanned,
};

use crate::{
    models::{EnumInfo, FieldInfo, FieldType, FieldWrapper, ItemInfo, StructInfo},
    utils::{as_turbofish, error, is_capnp_attr, try_peel_type},
};

impl ItemInfo {
    pub fn parse_input(input: &DeriveInput) -> Result<Self> {
        match &input.data {
            Data::Struct(struct_data) => Ok(Self::Struct(StructInfo::parse_struct(
                &input.ident,
                &input.generics,
                struct_data,
            )?)),
            Data::Enum(enum_data) => Ok(Self::Enum(EnumInfo::parse_enum(
                &input.ident,
                &input.generics,
                enum_data,
            )?)),
            Data::Union(union_data) => error(
                union_data.union_token.span(),
                "expected one of: `struct`, `enum`",
            ),
        }
    }
}

impl StructInfo {
    fn parse_struct(ident: &Ident, generics: &Generics, data: &DataStruct) -> Result<Self> {
        let ident = ident.clone();
        let fields = data
            .fields
            .iter()
            .map(FieldInfo::parse_field)
            .collect::<Result<Vec<FieldInfo>>>()?;

        let generics = generics
            .params
            .iter()
            .map(|generic| match generic {
                GenericParam::Type(generic) => Ok(generic.ident.clone()),
                _ => error(generic.span(), "invalid generic type"),
            })
            .collect::<Result<Vec<Ident>>>()?;

        Ok(Self {
            ident,
            fields,
            generics,
        })
    }
}

impl EnumInfo {
    fn parse_enum(ident: &Ident, generics: &Generics, data: &DataEnum) -> Result<Self> {
        let ident = ident.clone();
        let fields = data
            .variants
            .iter()
            .map(FieldInfo::parse_variant)
            .collect::<Result<Vec<FieldInfo>>>()?;

        let generics = generics
            .params
            .iter()
            .map(|generic| match generic {
                GenericParam::Type(generic) => Ok(generic.ident.clone()),
                _ => error(generic.span(), "invalid generic type"),
            })
            .collect::<Result<Vec<Ident>>>()?;

        Ok(Self {
            ident,
            fields,
            generics,
        })
    }
}

impl FieldInfo {
    fn parse_field(field: &Field) -> Result<Self> {
        let attr_info = FieldAttributesInfo::new(&field.attrs)?;
        let (field_type, field_wrapper) = FieldType::parse(&field.ty, attr_info.type_specifier)?;

        // Check for tuple struct field (ident is None) and error out if necessary
        let rust_name = match field.ident.as_ref() {
            Some(ident) => ident.clone(),
            None => {
                // This occurs for tuple struct fields like `struct Foo(i32);`
                // The current implementation doesn't handle tuple structs correctly.
                // Error out until support is added.
                return error(
                    field.span(), // Span of the whole field
                    "Tuple structs are not currently supported. Use a named struct instead.",
                );
            }
        };

        if matches!(field_type, FieldType::Phantom)
            && (attr_info.skip
                || attr_info.skip_read
                || attr_info.skip_write
                || attr_info.union_field
                || attr_info.default.is_some()
                || attr_info.name_override.is_some()
                || !matches!(
                    attr_info.type_specifier,
                    FieldAttributeTypeSpecifier::Default
                ))
        {
            return error(
                field.ty.span(),
                "PhantomData fields cannot have field attributes",
            );
        }

        let (is_union_field, is_optional, is_boxed) = match field_wrapper {
            FieldWrapper::Box(box_ident) if attr_info.union_field => {
                return error(box_ident.span(), "`Box<T>` types cannot be `union_field`s");
            }
            FieldWrapper::None if attr_info.union_field => {
                return error(field.ty.span(), "`union_field`s must be `Option<T>`");
            }
            FieldWrapper::Option(_) if attr_info.union_field => (true, false, false),
            FieldWrapper::Option(_) => (false, true, false),
            FieldWrapper::Box(_) => (false, false, true),
            FieldWrapper::None => (false, false, false),
        };

        let (skip_read, skip_write) = if attr_info.skip {
            (true, true)
        } else {
            (attr_info.skip_read, attr_info.skip_write)
        };

        match field_type {
            FieldType::UnnamedUnion(union_path) if is_union_field => {
                return error(union_path.span(), "unions cannot contain unnamed unions");
            }
            FieldType::GroupOrUnion(path) if is_optional => {
                return error(path.span(), "Groups and unions cannot be optional");
            }
            FieldType::UnnamedUnion(path) if is_optional => {
                return error(path.span(), "Groups and unions cannot be optional");
            }
            _ => {}
        }

        if is_boxed {
            todo!("`Box<T>`")
        }

        Ok(Self {
            rust_name, // Use the validated rust_name
            field_type,
            capnp_name_override: attr_info.name_override,
            has_phantom_in_variant: false,
            is_union_field,
            is_optional,
            _is_boxed: is_boxed,
            skip_read,
            skip_write,
            default_override: attr_info.default,
        })
    }
    fn parse_variant(variant: &Variant) -> Result<Self> {
        let (variant_type, is_phantom) = get_variant_type(&variant.fields)?;
        let attr_info = FieldAttributesInfo::new(&variant.attrs)?;
        let (field_type, field_wrapper) = match variant_type {
            Some(ty) => FieldType::parse(ty, attr_info.type_specifier)?,
            None => (FieldType::EnumVariant, FieldWrapper::None),
        };

        match field_type {
            FieldType::Phantom => {
                // If field_type is Phantom, it means the first type in the variant was PhantomData.
                // We use variant.fields.span() as variant_type must have been Some in this branch.
                return error(
                    variant.fields.span(),
                    "Enums may not have `PhantomData` in the first spot in their variants. \
                       Place them in the second slot.",
                );
            }
            FieldType::UnnamedUnion(_) => {
                // If field_type is UnnamedUnion, variant_type must have been Some.
                // Use variant.fields.span() for error location.
                return error(
                    variant.fields.span(),
                    "unions cannot contain unnamed unions.",
                );
            }
            _ => {}
        }

        if let FieldWrapper::Option(ident) = field_wrapper {
            return error(ident.span(), "Enums may not have `Option<T>`");
        }
        if attr_info.skip
            || attr_info.skip_read
            || attr_info.skip_write
            || attr_info.default.is_some()
            || attr_info.union_field
        {
            return error(
                variant.span(),
                "Enums variants cannot have `skip`, `default`, or `union_field` attributes.",
            );
        }

        if matches!(field_type, FieldType::EnumVariant)
            && !matches!(
                attr_info.type_specifier,
                FieldAttributeTypeSpecifier::Default
            )
        {
            return error(
                variant.span(),
                "Simple enums cannot have type specifier attributes",
            );
        }

        let is_boxed = matches!(field_wrapper, FieldWrapper::Box(_));

        if is_boxed {
            todo!("`Box<T>`")
        }

        Ok(Self {
            rust_name: variant.ident.clone(),
            field_type,
            capnp_name_override: attr_info.name_override,
            has_phantom_in_variant: is_phantom,
            is_union_field: false,
            is_optional: false,
            _is_boxed: is_boxed,
            skip_read: false,
            skip_write: false,
            default_override: None,
        })
    }
}

impl FieldType {
    fn parse(ty: &Type, specifier: FieldAttributeTypeSpecifier) -> Result<(Self, FieldWrapper)> {
        match try_peel_type(ty) {
            Some((ident, sub_type)) => match ident.to_string().as_str() {
                "PhantomData" => Ok((Self::Phantom, FieldWrapper::None)),
                "Option" => Ok((
                    Self::parse_type(sub_type, specifier)?,
                    FieldWrapper::Option(ident.clone()),
                )),
                "Box" => Ok((
                    Self::parse_type(sub_type, specifier)?,
                    FieldWrapper::Box(ident.clone()),
                )),
                _ => Ok((Self::parse_type(ty, specifier)?, FieldWrapper::None)),
            },
            None => Ok((Self::parse_type(ty, specifier)?, FieldWrapper::None)),
        }
    }

    fn parse_type(ty: &Type, specifier: FieldAttributeTypeSpecifier) -> Result<Self> {
        match ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(Self::Void()),
            Type::Path(path) => {
                let path = &path.path;
                // Use ok_or_else to handle potential empty path segments
                let last_segment = path.segments.last().ok_or_else::<syn::Error, _>(|| {
                    syn::Error::new(path.span(), "Type path must have at least one segment")
                })?;
                let ident = &last_segment.ident;

                if matches!(ident.to_string().as_str(), "Option" | "Box" | "PhantomData") {
                    // These are taken care of before this function is called
                    error(
                        ident.span(),
                        "invalid nested type (Option/Box/PhantomData should be handled externally)",
                    )
                } else if is_capnp_primative(path) {
                    Ok(Self::Primitive(path.clone()))
                } else if *ident == "String" {
                    Ok(Self::Text(path.clone()))
                } else if matches!(specifier, FieldAttributeTypeSpecifier::Data)
                    && is_capnp_data_type(path)
                {
                    // Specific check for `Vec<u8>` when `data` specifier is used
                    Ok(Self::Data(path.clone()))
                } else {
                    // Handle different path argument types and specifiers
                    match &last_segment.arguments {
                        PathArguments::None => match specifier {
                            // No type arguments, decision based on specifier
                            FieldAttributeTypeSpecifier::Default => Ok(Self::Struct(path.clone())),
                            FieldAttributeTypeSpecifier::EnumRemote => {
                                Ok(Self::EnumRemote(path.clone()))
                            }
                            FieldAttributeTypeSpecifier::Enum => Ok(Self::Enum(path.clone())),
                            FieldAttributeTypeSpecifier::GroupOrUnion => {
                                Ok(Self::GroupOrUnion(path.clone()))
                            }
                            FieldAttributeTypeSpecifier::UnnamedUnion => {
                                Ok(Self::UnnamedUnion(path.clone()))
                            }
                            FieldAttributeTypeSpecifier::Data => error(
                                path.span(), // Use path span for better error location
                                "fields with `data` attribute must be of type `Vec<u8>`",
                            ),
                        },
                        PathArguments::AngleBracketed(args) if ident == "Vec" => {
                            // Handle `Vec<T>` specifically
                            // Use pattern matching on slice to safely handle arguments
                            match args.args.iter().collect::<Vec<_>>().as_slice() {
                                [GenericArgument::Type(inner_ty)] => {
                                    Ok(Self::List(Box::new(Self::parse_type(inner_ty, specifier)?)))
                                }
                                [arg] => error(
                                    arg.span(),
                                    "invalid generic argument type for Vec: expected a type",
                                ),
                                [] => error(
                                    args.span(),
                                    "`Vec` fields must have exactly one type argument, found zero",
                                ),
                                _ => error(
                                    args.span(),
                                    "`Vec` fields must have exactly one type argument, found multiple",
                                ),
                            }
                        }
                        PathArguments::AngleBracketed(args) => {
                            // Handle generic types like `MyStruct<T>`
                            match specifier {
                                FieldAttributeTypeSpecifier::Default => {
                                    Ok(Self::GenericStruct(path.clone()))
                                }
                                FieldAttributeTypeSpecifier::GroupOrUnion => {
                                    Ok(Self::GroupOrUnion(path.clone()))
                                }
                                FieldAttributeTypeSpecifier::UnnamedUnion => {
                                    Ok(Self::UnnamedUnion(path.clone()))
                                }
                                // Other specifiers generally don't support generic arguments
                                _ => error(
                                    args.span(),
                                    "generic arguments are not supported with this type specifier",
                                ),
                            }
                        }
                        PathArguments::Parenthesized(args) => {
                            // Parenthesized arguments like `Fn(T) -> U` are not supported
                            error(
                                args.span(),
                                "parenthesized generic arguments (Fn traits) are not supported",
                            )
                        }
                    }
                }
            }
            _ => error(
                ty.span(),
                "incompatible field type (expected path or unit tuple)",
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum FieldAttributeTypeSpecifier {
    Default,
    EnumRemote,
    Enum,
    GroupOrUnion,
    UnnamedUnion,
    Data,
}

#[allow(clippy::struct_excessive_bools)]
struct FieldAttributesInfo {
    pub name_override: Option<Ident>,
    pub type_specifier: FieldAttributeTypeSpecifier,
    pub default: Option<Path>,
    pub skip: bool,
    pub skip_read: bool,
    pub skip_write: bool,
    pub union_field: bool,
}

impl FieldAttributesInfo {
    #[allow(clippy::too_many_lines)]
    pub fn new(attributes: &[Attribute]) -> Result<Self> {
        let mut attr_info = Self {
            name_override: None,
            type_specifier: FieldAttributeTypeSpecifier::Default,
            default: None,
            skip: false,
            skip_read: false,
            skip_write: false,
            union_field: false,
        };

        let mut processed_attrs = HashMap::new();
        for attr in attributes {
            if !is_capnp_attr(attr) {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let attr = if meta.path.is_ident("name") {
                    let name = meta.value()?.parse::<LitStr>()?.parse::<Ident>()?;

                    attr_info.name_override = Some(name);
                    FieldAttribute::Name(meta.path.clone())
                } else if meta.path.is_ident("type") {
                    let lit_str = meta.value()?.parse::<LitStr>()?.value();

                    match lit_str.as_str() {
            "enum" => {
              attr_info.type_specifier = FieldAttributeTypeSpecifier::Enum;
              FieldAttribute::Type(meta.path.clone())
            }
            "enum_remote" => {
              attr_info.type_specifier = FieldAttributeTypeSpecifier::EnumRemote;
              FieldAttribute::Type(meta.path.clone())
            }
            "group" | "union" => {
              attr_info.type_specifier = FieldAttributeTypeSpecifier::GroupOrUnion;
              FieldAttribute::Type(meta.path.clone())
            }
            "unnamed_union" => {
              attr_info.type_specifier = FieldAttributeTypeSpecifier::UnnamedUnion;
              FieldAttribute::Type(meta.path.clone())
            }
            "data" => {
              attr_info.type_specifier = FieldAttributeTypeSpecifier::Data;
              FieldAttribute::Type(meta.path.clone())
            }
            _ => {
              return Err(meta.error(
                "expected `enum`, `enum_remote`, `group`, `union`, `unnamed_union`, or `data`",
              ))
            }
          }
                } else if meta.path.is_ident("default") {
                    let path = meta.value()?.parse::<LitStr>()?.parse::<Path>()?;

                    if path == as_turbofish(&path) {
                        attr_info.default = Some(path);
                        FieldAttribute::Default(meta.path.clone())
                    } else {
                        return Err(meta.error("not in turbofish format"));
                    }
                } else if meta.path.is_ident("skip") {
                    attr_info.skip = true;
                    FieldAttribute::Skip(meta.path.clone())
                } else if meta.path.is_ident("skip_read") {
                    attr_info.skip_read = true;
                    FieldAttribute::SkipRead(meta.path.clone())
                } else if meta.path.is_ident("skip_write") {
                    attr_info.skip_write = true;
                    FieldAttribute::SkipWrite(meta.path.clone())
                } else if meta.path.is_ident("union_variant") {
                    attr_info.union_field = true;
                    FieldAttribute::UnionField(meta.path.clone())
                } else {
                    return Err(meta.error(
                        "expected `name`, `type`, `skip`, `skip_read`, \
                `skip_write`, `default`, or `union_variant`",
                    ));
                };

                let discriminant = discriminant(&attr);

                if let hash_map::Entry::Vacant(e) = processed_attrs.entry(discriminant) {
                    e.insert(attr);

                    Ok(())
                } else {
                    Err(meta.error("duplicate attribute"))
                }
            })?;
        }

        // Validate
        for attr in processed_attrs.values() {
            match attr {
                FieldAttribute::Default(ident)
                    if !processed_attrs.values().any(|a| {
                        matches!(a, FieldAttribute::Skip(_) | FieldAttribute::SkipRead(_))
                    }) =>
                {
                    return error(
                        ident.span(),
                        "`default` attribute with no `skip` or `skip_read` will never be used",
                    );
                }
                FieldAttribute::Skip(ident)
                    if processed_attrs.values().any(|a| {
                        matches!(
                            a,
                            FieldAttribute::SkipRead(_) | FieldAttribute::SkipWrite(_)
                        )
                    }) =>
                {
                    return error(
                        ident.span(),
                        "`skip` specified in additon to `skip_read` and/or `skip_write`",
                    );
                }
                _ => {}
            }
        }

        Ok(attr_info)
    }
}

#[derive(Debug, Clone)]
enum FieldAttribute {
    Name(Path),
    Type(Path),
    Default(Path),
    Skip(Path),
    SkipRead(Path),
    SkipWrite(Path),
    UnionField(Path),
}

impl ToTokens for FieldAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Type(a)
            | Self::Default(a)
            | Self::Skip(a)
            | Self::SkipRead(a)
            | Self::SkipWrite(a)
            | Self::UnionField(a)
            | Self::Name(a) => tokens.extend(a.into_token_stream()),
        }
    }
}

fn is_capnp_primative(path: &Path) -> bool {
    path.segments.last().map_or_else(
        || false,
        |last_segment| {
            matches!(
                last_segment.ident.to_string().as_str(),
                "bool"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "f32"
                    | "f64"
            )
        },
    )
}

/// Returns if the type signature is `Vec<u8>`, which corresponds to capnp's `Data`
fn is_capnp_data_type(path: &Path) -> bool {
    if let Some(last_segment) = path.segments.last() {
        if last_segment.ident == "Vec" {
            if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                if args.args.len() == 1 {
                    if let Some(GenericArgument::Type(Type::Path(inner_type_path))) =
                        args.args.first()
                    {
                        if let Some(inner_last_segment) = inner_type_path.path.segments.last() {
                            return inner_last_segment.ident == "u8";
                        }
                    }
                }
            }
        }
    }
    false
}

//returns the type of the variant's first slot, and true if the second slot contains PhntomData
fn get_variant_type(fields: &Fields) -> Result<(Option<&Type>, bool)> {
    match fields {
        Fields::Unit => Ok((None, false)),
        Fields::Unnamed(fields) => {
            match fields.unnamed.iter().collect::<Vec<_>>().as_slice() {
                // Enum variant with one field: E::Variant(T)
                [field] => Ok((Some(&field.ty), false)),
                // Enum variant with two fields: E::Variant(T, PhantomData<...>)
                [first_field, second_field] => {
                    let second_field_type = &second_field.ty;
                    match second_field_type {
                        Type::Path(path) => {
                            path.path.segments.last().map_or_else(|| error(path.span(), "internal error: Type::Path has no segments"), |last_segment| if last_segment.ident == "PhantomData" {
                                    Ok((Some(&first_field.ty), true))
                                } else {
                                    error(
                                        second_field_type.span(),
                                        "second type of an enum variant tuple can only be `PhantomData<T>`",
                                    )
                                })
                        }
                        _ => error(
                            second_field_type.span(),
                            "second type of an enum variant tuple can only be `PhantomData<T>`",
                        ),
                    }
                }
                // Enum variant with zero fields (e.g. `E::Variant()`) - treat like Unit
                [] => Ok((None, false)),
                // Enum variant with more than two fields
                _ => error(
                    fields.span(),
                    "enum variants may only contain 1 field (plus an optional `PhantomData`)",
                ),
            }
        }
        Fields::Named(_) => error(
            fields.span(),
            "named fields in enum variants are not supported yet. Use tuple variants like `Variant(T)` instead.",
        ), // Or `unimplemented!()` if preferred
    }
}

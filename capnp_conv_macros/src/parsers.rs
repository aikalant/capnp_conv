use std::{
    collections::{hash_map, HashMap},
    convert::TryFrom,
    mem::discriminant,
};

use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2, parse_str,
    spanned::Spanned,
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, GenericArgument,
    GenericParam, Generics, Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path, PathArguments,
    Result, Type, Variant,
};

use crate::{
    models::{EnumInfo, FieldInfo, FieldType, FieldWrapper, ItemInfo, StructInfo},
    utils::{as_turbofish, error, is_capnp_attr, try_peel_type},
};

impl ItemInfo {
    pub fn parse_input(input: &DeriveInput) -> Result<Self> {
        match &input.data {
            Data::Struct(struct_data) => Ok(ItemInfo::Struct(StructInfo::parse_struct(
                &input.ident,
                &input.generics,
                struct_data,
            )?)),
            Data::Enum(enum_data) => Ok(ItemInfo::Enum(EnumInfo::parse_enum(
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

        Ok(StructInfo {
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

        Ok(EnumInfo {
            ident,
            fields,
            generics,
        })
    }
}

impl FieldInfo {
    fn parse_field(field: &Field) -> Result<Self> {
        let attr_info = FieldAttributesInfo::new(&field.attrs)?;
        let (field_type, field_wrapper) = FieldType::parse(&field.ty, &attr_info.type_specifier)?;

        if let FieldType::Phantom = field_type {
            if attr_info.skip
                || attr_info.skip_read
                || attr_info.skip_write
                || attr_info.union_field
                || attr_info.default.is_some()
                || attr_info.name_override.is_some()
                || !matches!(
                    attr_info.type_specifier,
                    FieldAttributeTypeSpecifier::Default
                )
            {
                return error(
                    field.ty.span(),
                    "PhantomData fields cannot have field attributes",
                );
            }
        }

        let (is_union_field, is_optional, is_boxed) = match field_wrapper {
            FieldWrapper::Box(box_ident) if attr_info.union_field => {
                return error(box_ident.span(), "`Box<T>` types cannot be `union_field`s")
            }
            FieldWrapper::None if attr_info.union_field => {
                return error(field.ty.span(), "`union_field`s must be `Option<T>`")
            }
            FieldWrapper::Option(_) if attr_info.union_field => (true, false, false),
            FieldWrapper::Option(_) => (false, true, false),
            FieldWrapper::Box(_) => (false, false, true),
            FieldWrapper::None => (false, false, false),
        };

        let (skip_read, skip_write) = match attr_info.skip {
            true => (true, true),
            false => (attr_info.skip_read, attr_info.skip_write),
        };

        match field_type {
            FieldType::UnnamedUnion(union_path, _) if is_union_field => {
                return error(union_path.span(), "unions cannot contain unnamed unions")
            }
            FieldType::GroupOrUnion(path, _) if is_optional => {
                return error(path.span(), "Groups and unions cannot be optional")
            }
            FieldType::UnnamedUnion(path, _) if is_optional => {
                return error(path.span(), "Groups and unions cannot be optional")
            }
            _ => {}
        }

        if is_boxed {
            todo!("`Box<T>`")
        }

        Ok(FieldInfo {
            rust_name: field.ident.as_ref().unwrap().clone(),
            field_type,
            capnp_name_override: attr_info.name_override,
            has_phantom_in_variant: false,
            is_union_field,
            is_optional,
            is_boxed,
            skip_read,
            skip_write,
            default_override: attr_info.default,
        })
    }
    fn parse_variant(variant: &Variant) -> Result<Self> {
        let (variant_type, is_phantom) = get_variant_type(&variant.fields)?;
        let attr_info = FieldAttributesInfo::new(&variant.attrs)?;
        let (field_type, field_wrapper) = match variant_type {
            Some(ty) => FieldType::parse(ty, &attr_info.type_specifier)?,
            None => (FieldType::EnumVariant, FieldWrapper::None),
        };

        match  field_type {
      FieldType::Phantom => return error(variant_type.unwrap().span(), "Enums may not have `PhantomData` in the first spot in their variants. Place them in the second slot."),
      FieldType::UnnamedUnion(_, _) => return error(variant_type.unwrap().span(), "unions cannot contain unnamed unions."),
      _ => {}
    };

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

        Ok(FieldInfo {
            rust_name: variant.ident.clone(),
            field_type,
            capnp_name_override: attr_info.name_override,
            has_phantom_in_variant: is_phantom,
            is_union_field: false,
            is_optional: false,
            is_boxed,
            skip_read: false,
            skip_write: false,
            default_override: None,
        })
    }
}

impl FieldType {
    fn parse(ty: &Type, specifier: &FieldAttributeTypeSpecifier) -> Result<(Self, FieldWrapper)> {
        match try_peel_type(ty) {
            Some((ident, sub_type)) => match ident.to_string().as_str() {
                "PhantomData" => Ok((FieldType::Phantom, FieldWrapper::None)),
                "Option" => Ok((
                    FieldType::parse_type(sub_type, specifier)?,
                    FieldWrapper::Option(ident.clone()),
                )),
                "Box" => Ok((
                    FieldType::parse_type(sub_type, specifier)?,
                    FieldWrapper::Box(ident.clone()),
                )),
                _ => Ok((FieldType::parse_type(ty, specifier)?, FieldWrapper::None)),
            },
            None => Ok((FieldType::parse_type(ty, specifier)?, FieldWrapper::None)),
        }
    }
    fn parse_type(ty: &Type, specifier: &FieldAttributeTypeSpecifier) -> Result<Self> {
        match ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(FieldType::Void(tuple.clone())),
            Type::Path(path) => {
                let path = &path.path;
                let last_segment = path.segments.last().unwrap();
                let ident = &last_segment.ident;

                if matches!(ident.to_string().as_str(), "Option" | "Box" | "PhantomData") {
                    // These are taken care of in before this
                    error(ident.span(), "invalid generic argument type")
                } else if is_capnp_primative(path) {
                    Ok(FieldType::Primitive(path.clone()))
                } else if *ident == "String"
                    || (matches!(specifier, FieldAttributeTypeSpecifier::Data)
                        && is_capnp_data_type(path))
                {
                    Ok(FieldType::Blob(path.clone()))
                } else {
                    match &last_segment.arguments {
                        PathArguments::None => match specifier {
                            FieldAttributeTypeSpecifier::Default => {
                                Ok(FieldType::Struct(path.clone()))
                            }
                            FieldAttributeTypeSpecifier::EnumRemote => {
                                Ok(FieldType::EnumRemote(path.clone()))
                            }
                            FieldAttributeTypeSpecifier::Enum => Ok(FieldType::Enum(path.clone())),
                            FieldAttributeTypeSpecifier::GroupOrUnion => {
                                Ok(FieldType::GroupOrUnion(path.clone(), Vec::with_capacity(0)))
                            }
                            FieldAttributeTypeSpecifier::UnnamedUnion => {
                                Ok(FieldType::UnnamedUnion(path.clone(), Vec::with_capacity(0)))
                            }
                            FieldAttributeTypeSpecifier::Data => error(
                                ident.span(),
                                "fields with `data` attribute must be of type `Vec<u8>`",
                            ),
                        },
                        PathArguments::AngleBracketed(args) if ident == "Vec" => {
                            match args.args.len() {
                                1 => {
                                    let arg = args.args.first().unwrap();
                                    match arg {
                                        GenericArgument::Type(ty) => Ok(FieldType::List(Box::new(
                                            FieldType::parse_type(ty, specifier)?,
                                        ))),
                                        _ => error(arg.span(), "invalid generic argument type"),
                                    }
                                }
                                _ => error(args.span(), "`Vec` fields must have only one argument"),
                            }
                        }
                        PathArguments::AngleBracketed(args) => {
                            let generics = args
                                .args
                                .iter()
                                .map(|arg| match arg {
                                    GenericArgument::Type(ty) => {
                                        FieldType::parse_type(ty, specifier)
                                    }
                                    _ => error(arg.span(), "invalid generic argument type"),
                                })
                                .collect::<Result<Vec<FieldType>>>()?;

                            match specifier {
                                FieldAttributeTypeSpecifier::Default => {
                                    Ok(FieldType::GenericStruct(path.clone(), generics))
                                }
                                FieldAttributeTypeSpecifier::GroupOrUnion => {
                                    Ok(FieldType::GroupOrUnion(path.clone(), generics))
                                }
                                FieldAttributeTypeSpecifier::UnnamedUnion => {
                                    Ok(FieldType::UnnamedUnion(path.clone(), generics))
                                }
                                _ => error(
                                    args.span(),
                                    "generic arguments can not be specified in unions",
                                ),
                            }
                        }
                        PathArguments::Parenthesized(args) => {
                            error(args.span(), "invalid generic argument types")
                        }
                    }
                }
            }
            _ => error(ty.span(), "incompatible field type"),
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
        for attr in attributes.iter().filter(is_capnp_attr) {
            let attr = parse2::<FieldAttribute>(attr.tokens.clone())?;
            let discriminant = discriminant(&attr);
            if let hash_map::Entry::Vacant(e) = processed_attrs.entry(discriminant) {
                e.insert(attr.clone());
                match attr {
                    FieldAttribute::Name(_, ident) => attr_info.name_override = Some(ident.clone()),
                    FieldAttribute::Type(_, type_specifier) => {
                        attr_info.type_specifier = type_specifier;
                    }
                    FieldAttribute::Default(_, default_path) => {
                        attr_info.default = Some(default_path.clone())
                    }
                    FieldAttribute::Skip(_) => attr_info.skip = true,
                    FieldAttribute::SkipRead(_) => attr_info.skip_read = true,
                    FieldAttribute::SkipWrite(_) => attr_info.skip_write = true,
                    FieldAttribute::UnionField(_) => attr_info.union_field = true,
                }
            } else {
                return error(attr.span(), "duplicate attribute");
            }
        }

        // Validate
        for attr in processed_attrs.values() {
            match attr {
                FieldAttribute::Default(ident, _)
                    if !processed_attrs.values().any(|a| {
                        matches!(a, FieldAttribute::Skip(_) | FieldAttribute::SkipRead(_))
                    }) =>
                {
                    return error(
                        ident.span(),
                        "`default` attribute with no `skip` or `skip_read` will never be used",
                    )
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
                    )
                }
                _ => {}
            }
        }

        Ok(attr_info)
    }
}

#[derive(Debug, Clone)]
enum FieldAttribute {
    Name(MetaNameValue, Ident),
    Type(MetaNameValue, FieldAttributeTypeSpecifier),
    Default(MetaNameValue, Path),
    Skip(Path),
    SkipRead(Path),
    SkipWrite(Path),
    UnionField(Path),
}

impl Spanned for FieldAttribute {
    fn span(&self) -> Span {
        match self {
            FieldAttribute::Name(a, _) => a.span(),
            FieldAttribute::Type(a, _) => a.span(),
            FieldAttribute::Default(a, _) => a.span(),
            FieldAttribute::Skip(a) => a.span(),
            FieldAttribute::SkipRead(a) => a.span(),
            FieldAttribute::SkipWrite(a) => a.span(),
            FieldAttribute::UnionField(a) => a.span(),
        }
    }
}

impl Parse for FieldAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let meta = content.parse::<NestedMeta>()?;

        Ok(match meta {
            NestedMeta::Lit(lit) => FieldAttribute::try_from(lit)?,
            NestedMeta::Meta(meta) => match meta {
                Meta::Path(path) => FieldAttribute::try_from(path)?,
                Meta::NameValue(name_value) => FieldAttribute::try_from(name_value)?,
                Meta::List(list) => FieldAttribute::try_from(list)?,
            },
        })
    }
}

impl TryFrom<Lit> for FieldAttribute {
    type Error = syn::Error;
    fn try_from(lit: Lit) -> Result<Self> {
        error(
      lit.span(),
      "expected `name`, `type`, `skip`, `skip_read`, `skip_write`, `default`, or `union_variant`",
    )
    }
}

impl TryFrom<MetaList> for FieldAttribute {
    type Error = syn::Error;
    fn try_from(list: MetaList) -> Result<Self> {
        error(
      list.span(),
      "expected `name`, `type`, `skip`, `skip_read`, `skip_write`, `default`, or `union_variant`",
    )
    }
}

impl TryFrom<Path> for FieldAttribute {
    type Error = syn::Error;
    fn try_from(path: Path) -> Result<Self> {
        let ident = match path.get_ident() {
            Some(ident) => Ok(ident),
            None => error(
                path.span(),
                "expected `skip`, `skip_read`, `skip_write`, or `union_variant`",
            ),
        }?;

        match ident.to_string().as_str() {
            "skip" => Ok(FieldAttribute::Skip(path.clone())),
            "skip_read" => Ok(FieldAttribute::SkipRead(path.clone())),
            "skip_write" => Ok(FieldAttribute::SkipWrite(path.clone())),
            "union_variant" => Ok(FieldAttribute::UnionField(path.clone())),
            _ => error(
                ident.span(),
                "expected `skip`, `skip_read`, `skip_write`, or `union_variant`",
            ),
        }
    }
}

impl TryFrom<MetaNameValue> for FieldAttribute {
    type Error = syn::Error;
    fn try_from(name_value: MetaNameValue) -> Result<Self> {
        let ident = match name_value.path.get_ident() {
            Some(ident) => Ok(ident),
            None => error(name_value.path.span(), "expected `name`, `type`, `default`"),
        }?;

        let lit_span = name_value.lit.span();
        let lit_str = name_value
            .lit
            .to_token_stream()
            .to_string()
            .trim_matches('"')
            .to_owned();

        match ident.to_string().as_str() {
            "name" => {
                let mut ident = parse_str::<Ident>(&lit_str)?;
                ident.set_span(lit_span);
                Ok(FieldAttribute::Name(name_value, ident))
            }
            "type" => {
                let type_specifier = match lit_str.as_str() {
                    "enum" => Ok(FieldAttributeTypeSpecifier::Enum),
                    "enum_remote" => Ok(FieldAttributeTypeSpecifier::EnumRemote),
                    "group" => Ok(FieldAttributeTypeSpecifier::GroupOrUnion),
                    "union" => Ok(FieldAttributeTypeSpecifier::GroupOrUnion),
                    "unnamed_union" => Ok(FieldAttributeTypeSpecifier::UnnamedUnion),
                    "data" => Ok(FieldAttributeTypeSpecifier::Data),
                    _ => error(
                        lit_span,
                        r#"expected `"enum"`, `"enum_remote"`, `"group"`, `"union"`, `"unnamed_union"`, or `"data"` "#,
                    ),
                }?;
                Ok(FieldAttribute::Type(name_value.clone(), type_specifier))
            }
            "default" => {
                let path = parse_str::<Path>(&lit_str)?;
                match path == as_turbofish(&path) {
                    true => Ok(FieldAttribute::Default(name_value, path)),
                    false => error(lit_span, "not in turbofish format"),
                }
            }
            _ => error(ident.span(), "expected `name`, `type`, `default`"),
        }
    }
}

fn is_capnp_primative(path: &Path) -> bool {
    matches!(
        path.segments.last().unwrap().ident.to_string().as_str(),
        "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
    )
}

/// Returns if the type signature is `Vec<u8>`, which corresponds to capnp's `Data`
fn is_capnp_data_type(path: &Path) -> bool {
    if path.segments.last().unwrap().ident == "Vec" {
        if let PathArguments::AngleBracketed(args) = &path.segments.last().unwrap().arguments {
            if args.args.len() == 1 {
                if let GenericArgument::Type(Type::Path(path)) = args.args.first().unwrap() {
                    return path.path.segments.last().unwrap().ident == "u8";
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
        Fields::Unnamed(fields) => match fields.unnamed.len() {
            1 => Ok((Some(&fields.unnamed.first().unwrap().ty), false)),
            2 => {
                let second_field_type = &fields.unnamed[1].ty;
                match second_field_type {
                    Type::Path(path)
                        if path
                            .path
                            .segments
                            .last()
                            .unwrap()
                            .ident
                            .to_string()
                            .as_str()
                            == "PhantomData" => {}
                    _ => {
                        return error(
                            second_field_type.span(),
                            "second type of an enum can only be `PhantomData<T>`",
                        )
                    }
                };
                Ok((Some(&fields.unnamed.first().unwrap().ty), true))
            }
            _ => error(
                fields.span(),
                "enum variants may only contain 1 field (plus an optional `PhantomData`",
            ),
        },
        _ => unimplemented!(),
    }
}

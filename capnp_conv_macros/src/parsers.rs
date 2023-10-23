use std::{
    collections::{hash_map, HashMap, VecDeque},
    mem::discriminant,
};

use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    token::Comma,
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, GenericArgument,
    GenericParam, Generics, Lit, LitStr, Meta, MetaNameValue, Path, PathArguments, Result, Type,
    Variant,
};

use crate::{
    models::{EnumInfo, FieldInfo, FieldType, ItemInfo, StructInfo},
    utils::{error, is_capnp_attr, try_peel_type},
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
        let (field_type, wrappers) = FieldWrapper::peel(&field.ty);
        let field_type = FieldType::parse_type(field_type, attr_info.type_specifier)?;

        if attr_info.read_with.is_some() || attr_info.write_with.is_some() {
            return error(field.span(), "Custom readers/writers not supported yet");
        }

        let mut is_phantom = false;
        let mut is_optional = false;
        let mut is_boxed = false;
        let mut is_union_field = false;
        for wrapper in &wrappers {
            if matches!(wrapper, FieldWrapper::Option(_)) {
                if is_boxed {
                    return error(field.ty.span(), "Place Box inside of Option instead");
                }
                if attr_info.union_field {
                    is_union_field = true;
                } else {
                    is_optional = true;
                }
            } else if matches!(wrapper, FieldWrapper::Box(_)) {
                is_boxed = true;
            } else if matches!(wrapper, FieldWrapper::PhantomData(_)) {
                if wrappers.len() != 1 {
                    return error(
                        field.ty.span(),
                        "PhantomData fields cannot have Box or Option wrappers",
                    );
                }
                is_phantom = true;
            }
        }

        if attr_info.union_field && !is_union_field {
            return error(field.span(), "Union fields must have Option");
        }

        if is_phantom
            && (attr_info.skip
                || attr_info.skip_read
                || attr_info.skip_write
                || attr_info.union_field
                || attr_info.default.is_some()
                || attr_info.name_override.is_some()
                || attr_info.read_with.is_some()
                || attr_info.write_with.is_some()
                || !matches!(
                    attr_info.type_specifier,
                    FieldAttributeTypeSpecifier::Default
                ))
        {
            return error(
                field.span(),
                "PhantomData fields cannot have field attributes",
            );
        }

        let (skip_read, skip_write) = if attr_info.skip {
            (true, true)
        } else {
            (attr_info.skip_read, attr_info.skip_write)
        };

        if attr_info.read_with.is_some() && skip_read {
            return error(
                field.span(),
                "Cannot skip reads and specify read_with together",
            );
        }

        if attr_info.write_with.is_some() && skip_write {
            return error(
                field.span(),
                "Cannot skip writes and specify write_with together",
            );
        }

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
            read_override: attr_info.read_with,
            write_override: attr_info.write_with,
        })
    }
    fn parse_variant(variant: &Variant) -> Result<Self> {
        let (variant_type, is_phantom) = get_variant_type(&variant.fields)?;
        let attr_info = FieldAttributesInfo::new(&variant.attrs)?;
        let (field_type, wrappers) = match variant_type {
            Some(ty) => {
                let (field_type, wrappers) = FieldWrapper::peel(ty);
                let field_type = FieldType::parse_type(field_type, attr_info.type_specifier)?;
                (field_type, wrappers)
            }
            None => (FieldType::EnumVariant, vec![]),
        };

        if attr_info.read_with.is_some() || attr_info.write_with.is_some() {
            return error(variant.span(), "Custom readers/writers not supported yet");
        }

        let mut is_boxed = false;
        for wrapper in wrappers {
            if matches!(wrapper, FieldWrapper::PhantomData(_)) {
                return error(variant_type.unwrap().span(), "Enums may not have `PhantomData` in the first spot in their variants. Place them in the second slot.");
            }
            if matches!(wrapper, FieldWrapper::PhantomData(_)) {
                return error(
                    variant_type.unwrap().span(),
                    "Enums may not have `Option<T>`",
                );
            }
            if matches!(wrapper, FieldWrapper::Box(_)) {
                is_boxed = true;
            }
        }

        if matches!(field_type, FieldType::UnnamedUnion(..)) {
            return error(
                variant_type.unwrap().span(),
                "unions cannot contain unnamed unions.",
            );
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
            read_override: None,
            write_override: None,
        })
    }
}

impl FieldType {
    fn parse_type(ty: &Type, specifier: FieldAttributeTypeSpecifier) -> Result<Self> {
        match ty {
            Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(FieldType::Void(tuple.clone())),
            Type::Path(path) => {
                let path = &path.path;
                let last_segment = path.segments.last().unwrap();
                let ident = &last_segment.ident;

                if matches!(ident.to_string().as_str(), "Option" | "Box") {
                    // These are taken care of in before this
                    error(ident.span(), "invalid generic argument type")
                } else if matches!(ident.to_string().as_str(), "PhantomData") {
                    Ok(FieldType::Phantom)
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

#[derive(Debug)]
pub enum FieldWrapper {
    PhantomData(Ident),
    Box(Ident),
    Option(Ident),
}

impl FieldWrapper {
    fn peel(ty: &Type) -> (&Type, Vec<FieldWrapper>) {
        let mut wrappers = vec![];
        return (peel_wrappers(ty, &mut wrappers), wrappers);

        #[allow(clippy::items_after_statements)]
        fn peel_wrappers<'a>(ty: &'a Type, wrappers: &mut Vec<FieldWrapper>) -> &'a Type {
            match try_peel_type(ty) {
                Some((ident, sub_type)) => match ident.to_string().as_str() {
                    "PhantomData" => {
                        wrappers.push(FieldWrapper::PhantomData(ident.clone()));
                        ty
                    }
                    "Option" => {
                        wrappers.push(FieldWrapper::Option(ident.clone()));
                        return peel_wrappers(sub_type, wrappers);
                    }
                    "Box" => {
                        wrappers.push(FieldWrapper::Box(ident.clone()));
                        return peel_wrappers(sub_type, wrappers);
                    }
                    _ => ty,
                },
                None => ty,
            }
        }
    }
}

struct FieldAttributesInfo {
    pub name_override: Option<Ident>,
    pub type_specifier: FieldAttributeTypeSpecifier,
    pub default: Option<Path>,
    pub read_with: Option<(Path, bool)>,
    pub write_with: Option<(Path, bool)>,
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
            read_with: None,
            write_with: None,
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
                        attr_info.default = Some(default_path.clone());
                    }
                    FieldAttribute::Skip(_) => attr_info.skip = true,
                    FieldAttribute::SkipRead(_) => attr_info.skip_read = true,
                    FieldAttribute::SkipWrite(_) => attr_info.skip_write = true,
                    FieldAttribute::UnionField(_) => attr_info.union_field = true,
                    FieldAttribute::ReadWith {
                        path, is_ptr_type, ..
                    } => attr_info.read_with = Some((path, is_ptr_type)),
                    FieldAttribute::WriteWith {
                        path, is_ptr_type, ..
                    } => attr_info.write_with = Some((path, is_ptr_type)),
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
    Name(Span, Ident),
    Type(Span, FieldAttributeTypeSpecifier),
    Default(Span, Path),
    ReadWith {
        span: Span,
        path: Path,
        is_ptr_type: bool,
    },
    WriteWith {
        span: Span,
        path: Path,
        is_ptr_type: bool,
    },
    Skip(Span),
    SkipRead(Span),
    SkipWrite(Span),
    UnionField(Span),
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
            FieldAttribute::ReadWith { span, .. } => *span,
            FieldAttribute::WriteWith { span, .. } => *span,
        }
    }
}

fn extract_pair_str(content: &Meta) -> Option<(&Ident, &LitStr)> {
    if let Meta::NameValue(MetaNameValue { path, lit, .. }) = content {
        if let Some(left) = path.get_ident() {
            if let Lit::Str(right) = lit {
                return Some((left, right));
            }
        }
    }
    None
}
fn extract_pair_ident(content: &Meta) -> Option<(&Ident, Ident)> {
    if let Meta::NameValue(MetaNameValue { path, lit, .. }) = content {
        if let Some(left) = path.get_ident() {
            if let Lit::Str(right) = lit {
                if let Ok(right) = right.parse::<Ident>() {
                    return Some((left, right));
                }
            }
        }
    }
    None
}
fn check_ptr_type(arg: &Meta) -> Result<bool> {
    let mut is_ptr_type = false;
    if let Some((key, value)) = extract_pair_ident(arg) {
        if key.to_string().as_str() != "pointer_type" {
            return error(key.span(), "expected \"pointer_type\"");
        }
        match value.to_string().as_str() {
            "true" => is_ptr_type = true,
            "false" => {}
            _ => return error(value.span(), "expected \"true\" or \"false\""),
        }
    }
    Ok(is_ptr_type)
}

impl Parse for FieldAttribute {
    #[allow(clippy::too_many_lines)]
    fn parse(input: ParseStream) -> Result<Self> {
        const UNKNOWN_KEY_MSG: &str = r#"expected `name`, `type`, `skip`, 
      `skip_read`, `skip_write`, `default`, `read_with`, 
      `write_with`, or `union_variant`"#;
        const UNKNOWN_TYPE_MSG: &str = r#"expected `"enum"`, `"enum_remote"`, `"group"`, 
      `"union"`, `"unnamed_union"`, or `"data"`"#;
        const TOO_MANY_EXPRS_MSG: &str = r#"Only one field attribute macro allowed per line"#;
        const EXPECTED_FUNCTION: &str = "Expected function";

        let content;
        parenthesized!(content in input);

        let args = content.parse_terminated::<Meta, Comma>(Meta::parse)?;
        let mut args_list = VecDeque::from_iter(&args);
        if let Some(arg) = args_list.pop_front() {
            if let Some((key, value)) = extract_pair_str(arg) {
                match key.to_string().as_str() {
                    "default" => {
                        if let Some(next) = args_list.pop_front() {
                            return error(next.span(), TOO_MANY_EXPRS_MSG);
                        }
                        if let Ok(path) = value.parse::<Path>() {
                            return Ok(FieldAttribute::Default(key.span(), path));
                        }
                        return error(value.span(), EXPECTED_FUNCTION);
                    }
                    "name" => {
                        if let Some(next) = args_list.pop_front() {
                            return error(next.span(), TOO_MANY_EXPRS_MSG);
                        }
                        if let Ok(ident) = value.parse::<Ident>() {
                            return Ok(FieldAttribute::Name(key.span(), ident));
                        }
                        return error(value.span(), EXPECTED_FUNCTION);
                    }
                    "read_with" => {
                        let is_ptr_type = if let Some(arg) = args_list.pop_back() {
                            if let Some(next) = args_list.pop_back() {
                                return error(next.span(), TOO_MANY_EXPRS_MSG);
                            }
                            check_ptr_type(arg)?
                        } else {
                            false
                        };
                        if let Ok(path) = value.parse::<Path>() {
                            return Ok(FieldAttribute::ReadWith {
                                span: key.span(),
                                path,
                                is_ptr_type,
                            });
                        }
                        return error(value.span(), EXPECTED_FUNCTION);
                    }
                    "write_with" => {
                        let is_ptr_type = if let Some(arg) = args_list.pop_back() {
                            if let Some(next) = args_list.pop_back() {
                                return error(next.span(), TOO_MANY_EXPRS_MSG);
                            }
                            check_ptr_type(arg)?
                        } else {
                            false
                        };
                        if let Ok(path) = value.parse::<Path>() {
                            return Ok(FieldAttribute::WriteWith {
                                span: key.span(),
                                path,
                                is_ptr_type,
                            });
                        }
                        return error(value.span(), EXPECTED_FUNCTION);
                    }
                    "type" => {
                        if let Some(next) = args_list.pop_front() {
                            return error(next.span(), TOO_MANY_EXPRS_MSG);
                        }
                        let type_specifier = match value
                            .to_token_stream()
                            .to_string()
                            .as_str()
                            .trim_matches('"')
                        {
                            "enum" => FieldAttributeTypeSpecifier::Enum,
                            "enum_remote" => FieldAttributeTypeSpecifier::EnumRemote,
                            "group" => FieldAttributeTypeSpecifier::GroupOrUnion,
                            "union" => FieldAttributeTypeSpecifier::GroupOrUnion,
                            "unnamed_union" => FieldAttributeTypeSpecifier::UnnamedUnion,
                            "data" => FieldAttributeTypeSpecifier::Data,
                            _ => return error(value.span(), UNKNOWN_TYPE_MSG),
                        };
                        return Ok(FieldAttribute::Type(key.span(), type_specifier));
                    }
                    _ => {
                        return error(key.span(), UNKNOWN_KEY_MSG);
                    }
                }
            } else if let Meta::Path(path) = arg {
                if let Some(next) = args_list.pop_front() {
                    return error(next.span(), TOO_MANY_EXPRS_MSG);
                }
                match path.to_token_stream().to_string().as_str() {
                    "skip" => return Ok(FieldAttribute::Skip(path.span())),
                    "skip_read" => return Ok(FieldAttribute::SkipRead(path.span())),
                    "skip_write" => return Ok(FieldAttribute::SkipWrite(path.span())),
                    "union_variant" => return Ok(FieldAttribute::UnionField(path.span())),
                    _ => {
                        return error(path.span(), UNKNOWN_KEY_MSG);
                    }
                }
            }
        }
        error(args.span(), UNKNOWN_KEY_MSG)
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
        Fields::Named(_) => unimplemented!(),
    }
}

use proc_macro2::Ident;
use syn::Path;

#[derive(Debug)]
pub enum ItemInfo {
    Struct(StructInfo),
    Enum(EnumInfo),
}

#[derive(Debug)]
pub struct StructInfo {
    pub ident: Ident,
    pub fields: Vec<FieldInfo>,
    pub generics: Vec<Ident>,
}

#[derive(Debug)]
pub struct EnumInfo {
    pub ident: Ident,
    pub fields: Vec<FieldInfo>,
    pub generics: Vec<Ident>,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub rust_name: Ident,
    pub field_type: FieldType,
    pub capnp_name_override: Option<Ident>,
    pub has_phantom_in_variant: bool,
    pub is_union_field: bool,
    pub is_optional: bool,
    pub _is_boxed: bool,
    pub skip_read: bool,
    pub skip_write: bool,
    pub default_override: Option<Path>,
}

#[derive(Debug)]
pub enum FieldType {
    Phantom,
    /// Only for capnp enums
    EnumVariant,
    /// ()
    Void(),
    /// bool, i8/16/32/64, u8/16/32/64, f32/64
    Primitive(Path),
    /// Vec<u8>
    Data(Path),
    /// String
    Text(Path),
    /// Non-generic capnp structs
    Struct(Path),
    /// Requires field attribute `#[capnp_conv(type = "enum")]`
    /// Indicates to use the pre-existing capnp code generated enum
    Enum(Path),
    /// Requires field attribute `#[capnp_conv(type = "enum_remote")]`
    /// Indicates to use the a manually defined enum
    EnumRemote(Path),
    /// Requires field attribute `#[capnp_conv(type = "group")]` or `#[capnp_conv(type = "union")]`
    /// Applys to named unions only
    /// These don't need to be unwrapped by readers
    GroupOrUnion(Path),
    /// Requires field attribute `#[capnp_conv(type = "unnamed_union")]`
    /// Reader/writer acts as a "passthrough", not needing to get/init anything
    UnnamedUnion(Path),
    /// Vec<T>
    List(Box<FieldType>),
    /// CapnpStruct(T1, T2, ...)
    GenericStruct(Path),
}

#[derive(Debug)]
pub enum FieldWrapper {
    None,
    Box(Ident),
    Option(Ident),
}

use std::marker::PhantomData;

use capnp_conv::capnp_conv;

use super::common_capnp::{self, basic_struct, comprehensive_struct, generic_struct};

#[capnp_conv(comprehensive_struct)]
#[derive(Debug, Clone, PartialEq)]
pub struct ComprehensiveStruct<T, Y> {
    pub void_val: (),
    pub bool_val: bool,
    pub i8_val: i8,
    pub i16_val: i16,
    pub i32_val: i32,
    pub i64_val: i64,
    pub u8_val: u8,
    pub u16_val: u16,
    pub u32_val: u32,
    pub u64_val: u64,
    pub f32_val: f32,
    pub f64_val: f64,
    pub text_val: String,
    #[capnp_conv(type = "data")]
    pub data_val: Vec<u8>,
    pub u8_list_val: Vec<u8>,
    pub nested_val: BasicStruct,
    pub list_val: Vec<Vec<BasicStruct>>,
    #[capnp_conv(type = "enum")]
    pub enum_val: common_capnp::ComprehensiveStructEnum,
    #[capnp_conv(type = "enum_remote")]
    pub enum_val_remote: ComprehensiveStructEnum,
    #[capnp_conv(type = "group")]
    pub group_val: ComprehensiveStructGroup<T, Y>,
    #[capnp_conv(type = "union")]
    pub union_val: ComprehensiveStructUnion<T, Y>,
    #[capnp_conv(type = "unnamed_union")]
    pub unnamed_union: ComprehensiveStructUnnamedUnion<T, Y>,
    pub t_val: T,
    pub y_val: Y,
    #[capnp_conv(type = "union")]
    pub comprehensive_union: ComprehensiveUnion<T, Y>,
    pub generic_val: GenericStruct<BasicStruct, BasicStruct>,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: (),
}

#[capnp_conv(comprehensive_struct::nested_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedStruct<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(generic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericStruct<A, B> {
    pub a_val: A,
    pub b_val: B,
}

#[capnp_conv(common_capnp::ComprehensiveStructEnum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComprehensiveStructEnum {
    Val1,
    Val2,
}

#[capnp_conv(comprehensive_struct::group_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComprehensiveStructGroup<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(comprehensive_struct::comprehensive_union::group_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComprehensiveUnionGroup<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(comprehensive_struct::comprehensive_union::union_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveUnionUnion<T, Y> {
    TVal(T),
    YVal(Y),
    #[capnp_conv(name = "GenericVal")]
    StopComplainingClippy(GenericStruct<BasicStruct, BasicStruct>),
}

#[capnp_conv(comprehensive_struct::union_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveStructUnion<T, Y> {
    TVal(T),
    YVal(Y),
}

#[capnp_conv(comprehensive_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveStructUnnamedUnion<T, Y> {
    TVal2(T, PhantomData<*const Y>),
    YVal2(()),
}

#[capnp_conv(comprehensive_struct::comprehensive_union)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveUnion<T, Y> {
    VoidVal(()),
    BoolVal(bool),
    I8Val(i8),
    TextVal(String),
    #[capnp_conv(type = "data")]
    DataVal(Vec<u8>),
    TVal(T),
    YVal(Y),
    ListVal(Vec<Vec<BasicStruct>>),
    #[capnp_conv(type = "enum")]
    EnumVal(common_capnp::ComprehensiveStructEnum),
    #[capnp_conv(type = "enum_remote")]
    EnumValRemote(ComprehensiveStructEnum),
    NestedVal(BasicStruct),
    #[capnp_conv(type = "group")]
    GroupVal(ComprehensiveUnionGroup<T, Y>),
    #[capnp_conv(type = "union")]
    UnionVal(ComprehensiveUnionUnion<T, Y>),
    GenericVal(GenericStruct<BasicStruct, BasicStruct>),
}

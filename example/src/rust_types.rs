use capnp_conv::capnp_conv;

use crate::example_capnp as capnp_types;

#[capnp_conv(capnp_types::basic_struct)]
#[derive(Clone, PartialEq)]
pub struct BasicStruct {
    pub val: i32,
}

#[capnp_conv(capnp_types::example_struct)]
#[derive(PartialEq)]
pub struct ExampleStruct<T> {
    pub i32_val: i32,
    pub text_val: String,
    #[capnp_conv(type = "data")]
    pub data_val: Vec<u8>,
    pub nested_val: BasicStruct,

    #[capnp_conv(type = "enum")]
    pub enum_val: capnp_types::ExampleEnum,

    #[capnp_conv(type = "enum_remote")]
    pub enum_val_remote: RemoteExampleEnum,

    pub generic_struct: GenericStruct<BasicStruct>,
    pub generic_generic_struct: GenericStruct<T>,
    pub list_val: Vec<Vec<GenericStruct<T>>>,

    #[capnp_conv(type = "group")]
    pub group_val: ExampleGroup<T>,

    #[capnp_conv(type = "union")]
    pub union_val: ExampleUnion<T>,

    #[capnp_conv(type = "unnamed_union")]
    pub unnamed_union: ExampleUnnamedUnion<T>,
}

#[capnp_conv(capnp_types::generic_struct)]
#[derive(Clone, PartialEq)]
pub struct GenericStruct<T> {
    pub val: T,
}

#[capnp_conv(capnp_types::ExampleEnum)]
#[derive(PartialEq)]
pub enum RemoteExampleEnum {
    Val1,
    Val2,
}

#[capnp_conv(capnp_types::example_struct::group_val)]
#[derive(PartialEq)]
pub struct ExampleGroup<T> {
    pub val1: T,
    pub val2: T,
}

#[capnp_conv(capnp_types::example_struct::union_val)]
#[derive(PartialEq)]
pub enum ExampleUnion<T> {
    Val1(T),
    Val2(T),
}

#[capnp_conv(capnp_types::example_struct)]
#[derive(PartialEq)]
pub enum ExampleUnnamedUnion<T> {
    Val1(T),
    Val2(T),
}

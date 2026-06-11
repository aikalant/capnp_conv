use capnp_conv::capnp_conv;

use super::non_exhaustive_capnp::{self, TestEnum};

#[capnp_conv(non_exhaustive_capnp::test_union, non_exhaustive)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestUnion {
    pub prim: i32,
    #[capnp_conv(union_variant)]
    pub union_val1: Option<()>,
    #[capnp_conv(union_variant)]
    pub union_val2: Option<()>,
}

#[capnp_conv(non_exhaustive_capnp::test_union_pure, non_exhaustive)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestUnionPure {
    #[capnp_conv(union_variant)]
    pub union_val1: Option<()>,
    #[capnp_conv(union_variant)]
    pub union_val2: Option<()>,
}

#[capnp_conv(non_exhaustive_capnp::TestEnum, non_exhaustive)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEnumRemote {
    Val1,
    Val2,
}

#[allow(clippy::struct_field_names)]
#[capnp_conv(non_exhaustive_capnp::test_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStruct {
    #[capnp_conv(type = "enum")]
    pub test_enum: TestEnum,
    #[capnp_conv(type = "enum_remote")]
    pub test_enum_remote: TestEnumRemote,
    pub test_union: TestUnion,
    pub test_union_pure: TestUnionPure,
}

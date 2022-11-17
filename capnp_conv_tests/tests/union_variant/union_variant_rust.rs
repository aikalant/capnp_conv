use capnp_conv::capnp_conv;

use super::union_variant_capnp::{basic_struct, union_struct, union_struct_pure};

#[capnp_conv(union_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionStruct {
    pub prim: i32,
    #[capnp_conv(union_variant)]
    pub union_val1: Option<String>,
    #[capnp_conv(union_variant)]
    pub union_val2: Option<BasicStruct>,
}

#[capnp_conv(union_struct_pure)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionStructPure {
    #[capnp_conv(union_variant)]
    pub union_val1: Option<String>,
    #[capnp_conv(union_variant)]
    pub union_val2: Option<BasicStruct>,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: i32,
}

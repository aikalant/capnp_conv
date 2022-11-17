use capnp_conv::capnp_conv;

use super::optional_capnp::{basic_struct, test_defaults, test_optional};

#[capnp_conv(test_optional)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestOptional {
    pub prim: Option<i32>,
    pub struc: Option<BasicStruct>,
    pub text: Option<String>,
    #[capnp_conv(type = "data")]
    pub data: Option<Vec<u8>>,
    pub list: Option<Vec<i32>>,
}

#[capnp_conv(test_defaults)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDefaults {
    pub prim: i32,
    pub struc: BasicStruct,
    pub text: String,
    #[capnp_conv(type = "data")]
    pub data: Vec<u8>,
    pub list: Vec<i32>,
}

#[capnp_conv(test_defaults)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDefaultsOptional {
    pub prim: Option<i32>,
    pub struc: Option<BasicStruct>,
    pub text: Option<String>,
    #[capnp_conv(type = "data")]
    pub data: Option<Vec<u8>>,
    pub list: Option<Vec<i32>>,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: i32,
}

use capnp_conv::capnp_conv;

use crate::test_capnp::{basic_struct, test_defaults, test_optional};

#[capnp_conv(test_optional)]
#[derive(Debug, Clone, PartialEq)]
pub struct TestOptional {
    pub prim: Option<i32>,
    pub text: Option<String>,
    #[capnp_conv(type = "data")]
    pub data: Option<Vec<u8>>,
    pub list: Option<Vec<i32>>,
    pub struc: Option<BasicStruct>,
}

#[capnp_conv(test_defaults)]
#[derive(Debug, Clone, PartialEq)]
pub struct TestDefaults {
    pub prim: i32,
    pub text: String,
    #[capnp_conv(type = "data")]
    pub data: Vec<u8>,
    pub list: Vec<i32>,
    pub struc: BasicStruct,
}

#[capnp_conv(test_defaults)]
#[derive(Debug, Clone, PartialEq)]
pub struct TestDefaultsOptional {
    pub prim: Option<i32>,
    pub text: Option<String>,
    #[capnp_conv(type = "data")]
    pub data: Option<Vec<u8>>,
    pub list: Option<Vec<i32>>,
    pub struc: Option<BasicStruct>,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq)]
pub struct BasicStruct {
    pub val: (),
}

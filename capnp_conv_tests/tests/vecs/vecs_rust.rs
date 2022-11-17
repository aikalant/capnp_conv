use capnp_conv::capnp_conv;

use super::vecs_capnp::{self, basic_struct, data_test_struct, wrapper_struct};

#[capnp_conv(wrapper_struct)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct WrapperStruct {
    pub primitive_list: Vec<i32>,
    pub struct_list: Vec<BasicStruct>,
    pub text_list: Vec<String>,
    #[capnp_conv(type = "data")]
    pub data_list: Vec<Vec<u8>>,
    #[capnp_conv(type = "enum")]
    pub enum_list: Vec<vecs_capnp::BasicEnum>,
    #[capnp_conv(type = "enum_remote")]
    pub enum_remote_list: Vec<BasicEnum>,
    //pub list_list: Vec<Vec<i32>,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: i32,
}

#[capnp_conv(vecs_capnp::BasicEnum)]
#[derive(Debug, PartialEq, Eq)]
pub enum BasicEnum {
    Val1,
    Val2,
}

#[capnp_conv(data_test_struct)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DataTestStruct {
    pub u8_list: Vec<u8>,
    #[capnp_conv(type = "data")]
    pub data_list: Vec<Vec<u8>>,
}

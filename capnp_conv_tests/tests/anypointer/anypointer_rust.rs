use capnp_conv::capnp_conv;

use super::anypointer_capnp::{any_pointer_test_struct, basic_struct};

#[capnp_conv(any_pointer_test_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyPointerTestStruct {
    pub prim: i32,
    pub struc: BasicStruct,
    //pub anypointer: BasicStruct,
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: i32,
}

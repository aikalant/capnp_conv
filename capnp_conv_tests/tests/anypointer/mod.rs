use capnp::message::TypedBuilder;

use self::anypointer_capnp::any_pointer_test_struct;

#[allow(unused, dead_code, clippy::all)]
pub mod anypointer_capnp;
mod anypointer_rust;

#[test]
fn anypointer_test() {
    // let mut builder = TypedBuilder::<any_pointer_test_struct::Owned>::new_default();
    // let mut root = builder.init_root();
    // let mut struct_builder = root.reborrow().init_struc();
    // let mut anypointer_builder = root.reborrow().init_any_pointer();
}

mod rust_types;
pub mod example_capnp {
    include!(concat!(env!("OUT_DIR"), "/", "example_capnp.rs"));
}

use std::vec;

use capnp::message::TypedBuilder;
use capnp_conv::{Readable, Writable};
use example_capnp as capnp_types;

#[allow(clippy::print_stdout)]
fn main() {
    let basic_struct = rust_types::BasicStruct { val: 10 };
    let generic_struct = rust_types::GenericStruct::<rust_types::BasicStruct> {
        val: basic_struct.clone(),
    };

    let input = rust_types::ExampleStruct::<rust_types::BasicStruct> {
        i32_val: 5,
        text_val: "hello".to_owned(),
        data_val: vec![0, 1, 2, 3],
        nested_val: basic_struct.clone(),
        enum_val: capnp_types::ExampleEnum::Val2,
        enum_val_remote: rust_types::RemoteExampleEnum::Val2,
        generic_struct: generic_struct.clone(),
        generic_generic_struct: generic_struct.clone(),
        list_val: vec![
            vec![generic_struct.clone()],
            vec![generic_struct.clone(), generic_struct.clone()],
        ],
        group_val: rust_types::ExampleGroup {
            val1: basic_struct.clone(),
            val2: basic_struct.clone(),
        },
        union_val: rust_types::ExampleUnion::Val2(basic_struct.clone()),
        unnamed_union: rust_types::ExampleUnnamedUnion::Val2(basic_struct.clone()),
    };

    let mut builder = TypedBuilder::<
        capnp_types::example_struct::Owned<capnp_types::basic_struct::Owned>,
    >::new_default();

    input.write(builder.init_root());

    let reader = builder.get_root_as_reader().unwrap();

    let output = rust_types::ExampleStruct::<rust_types::BasicStruct>::read(reader).unwrap();

    println!("Input == Output: {}", input == output);
}

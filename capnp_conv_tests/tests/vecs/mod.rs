#[allow(unused, dead_code, clippy::all)]
pub mod vecs_capnp;
mod vecs_rust;

use capnp::message::TypedBuilder;
use capnp_conv::{
    ReadableList, ReadablePrimitiveList, ReadableRemoteEnumList, WritableList,
    WritablePrimitiveList, WritableRemoteEnumList,
};
use vecs_rust::BasicStruct;

use crate::vecs::{
    vecs_capnp::{data_test_struct, wrapper_struct},
    vecs_rust::BasicEnum,
};

#[test]
fn test_primitive() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input = vec![0, 1, 2, 3, 4];

    builder
        .init_root()
        .init_primitive_list(input.len() as u32)
        .write(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_primitive_list()
        .unwrap()
        .read()
        .unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_primitive_conv() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input: Vec<i64> = vec![0, 1, 2, 3, 4];

    builder
        .init_root()
        .init_primitive_list(input.len() as u32)
        .try_write(&input)
        .unwrap();

    let output: Vec<i16> = builder
        .get_root_as_reader()
        .unwrap()
        .get_primitive_list()
        .unwrap()
        .try_read()
        .unwrap();

    let output: Vec<i64> = output.iter().map(|i| (*i).into()).collect();

    assert_eq!(input, output);
}

#[test]
fn test_data() {
    let mut builder = TypedBuilder::<data_test_struct::Owned>::new_default();

    let input: Vec<Vec<u8>> = vec![vec![0, 1, 2], vec![3, 4, 5], vec![6, 7, 8]];
    builder
        .init_root()
        .init_data_list(input.len() as u32)
        .write(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_data_list()
        .unwrap()
        .read()
        .unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_text() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input = vec!["one", "two", "three"];
    builder
        .init_root()
        .init_text_list(input.len() as u32)
        .write(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_text_list()
        .unwrap()
        .read()
        .unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_struct() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input = vec![
        BasicStruct { val: 0 },
        BasicStruct { val: 1 },
        BasicStruct { val: 2 },
    ];
    builder
        .init_root()
        .init_struct_list(input.len() as u32)
        .write(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_struct_list()
        .unwrap()
        .read()
        .unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_enum() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input = vec![vecs_capnp::BasicEnum::Val2, vecs_capnp::BasicEnum::Val1];
    builder
        .init_root()
        .init_enum_list(input.len() as u32)
        .write(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_enum_list()
        .unwrap()
        .read()
        .unwrap();

    assert_eq!(input, output);
}

#[test]
fn test_remote_enum() {
    let mut builder = TypedBuilder::<wrapper_struct::Owned>::new_default();

    let input = vec![BasicEnum::Val2, BasicEnum::Val1];
    builder
        .init_root()
        .init_enum_remote_list(input.len() as u32)
        .write_remote(&input);

    let output = builder
        .get_root_as_reader()
        .unwrap()
        .get_enum_remote_list()
        .unwrap()
        .read_remote()
        .unwrap();

    assert_eq!(input, output);
}

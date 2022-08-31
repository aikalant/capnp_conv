mod test_rust;

use std::marker::PhantomData;

use capnp::message::TypedBuilder;
use capnp_conv::{Readable, Writeable};

use crate::{
    common::test_rust::{
        BasicStruct, ComprehensiveStruct, ComprehensiveStructEnum, ComprehensiveStructGroup,
        ComprehensiveStructUnion, ComprehensiveStructUnnamedUnion, ComprehensiveUnion,
        GenericStruct,
    },
    test_capnp::{basic_struct, comprehensive_struct},
};

#[test]
pub fn check() {
    let basic_struct = BasicStruct { val: () };
    let input = ComprehensiveStruct::<BasicStruct, BasicStruct> {
        void_val: (),
        bool_val: true,
        i8_val: 1,
        i16_val: 2,
        i32_val: 3,
        i64_val: 4,
        u8_val: 5,
        u16_val: 6,
        u32_val: 7,
        u64_val: 8,
        f32_val: 9.9,
        f64_val: 10.1,
        text_val: "hello".to_owned(),
        data_val: vec![1, 2, 3, 4, 5],
        u8_list_val: vec![5, 4, 3, 2, 1],
        nested_val: basic_struct.clone(),
        list_val: vec![vec![basic_struct.clone()]],
        enum_val: crate::test_capnp::ComprehensiveStructEnum::Val2,
        enum_val_remote: ComprehensiveStructEnum::Val2,
        group_val: ComprehensiveStructGroup {
            t_val: basic_struct.clone(),
            y_val: basic_struct.clone(),
        },
        union_val: ComprehensiveStructUnion::YVal(basic_struct.clone()),
        unnamed_union: ComprehensiveStructUnnamedUnion::<BasicStruct, BasicStruct>::TVal2(
            basic_struct.clone(),
            PhantomData,
        ),
        t_val: basic_struct.clone(),
        y_val: basic_struct.clone(),
        comprehensive_union: ComprehensiveUnion::TextVal("hi".to_owned()),
        generic_val: GenericStruct {
            a_val: basic_struct.clone(),
            b_val: basic_struct,
        },
    };

    let mut builder = TypedBuilder::<
        comprehensive_struct::Owned<basic_struct::Owned, basic_struct::Owned>,
    >::new_default();
    input.write(builder.init_root()).unwrap();
    let reader = builder.get_root_as_reader().unwrap();

    let output = ComprehensiveStruct::<BasicStruct, BasicStruct>::read(reader).unwrap();

    assert!(input == output)
}

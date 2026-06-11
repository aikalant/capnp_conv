#[allow(unused, clippy::all, clippy::pedantic)]
#[rustfmt::skip]
mod non_exhaustive_capnp;
mod non_exhaustive_rust;

use capnp::message::TypedBuilder;
use capnp_conv::Readable;
use non_exhaustive_capnp::{test_struct, TestEnum};
use non_exhaustive_rust::{TestEnumRemote, TestStruct, TestUnion, TestUnionPure};

use crate::assert_identical;

#[test]
fn non_exhaustive_test() {
    assert_identical(&TestStruct {
        test_enum: TestEnum::Val1,
        test_enum_remote: TestEnumRemote::Val1,
        test_union: TestUnion {
            prim: 1,
            union_val1: Some(()),
            union_val2: None,
        },
        test_union_pure: TestUnionPure {
            union_val1: Some(()),
            union_val2: None,
        },
    });

    assert!(TestEnumRemote::try_from(TestEnum::Extra).is_err());

    let mut builder = TypedBuilder::<test_struct::Owned>::new_default();

    {
        let mut root = builder.init_root();

        root.reborrow().set_test_enum(TestEnum::Val1);

        root.reborrow().set_test_enum_remote(TestEnum::Extra);

        let mut test_union = root.reborrow().init_test_union();
        test_union.reborrow().set_union_val1(());

        let mut test_union_pure = root.reborrow().init_test_union_pure();
        test_union_pure.reborrow().set_union_val1(());

        assert!(TestStruct::read(builder.get_root_as_reader().unwrap()).is_err());
    }

    {
        let mut root = builder.init_root();

        root.reborrow().set_test_enum(TestEnum::Val1);

        root.reborrow().set_test_enum_remote(TestEnum::Val1);

        let mut test_union = root.reborrow().init_test_union();
        test_union.reborrow().set_extra(());

        let mut test_union_pure = root.reborrow().init_test_union_pure();
        test_union_pure.reborrow().set_union_val1(());

        assert!(TestStruct::read(builder.get_root_as_reader().unwrap()).is_err());
    }

    {
        let mut root = builder.init_root();

        root.reborrow().set_test_enum(TestEnum::Val1);

        root.reborrow().set_test_enum_remote(TestEnum::Val1);

        let mut test_union = root.reborrow().init_test_union();
        test_union.reborrow().set_union_val1(());

        let mut test_union_pure = root.reborrow().init_test_union_pure();
        test_union_pure.reborrow().set_extra(());

        assert!(TestStruct::read(builder.get_root_as_reader().unwrap()).is_err());
    }

    {
        let mut root = builder.init_root();

        root.reborrow().set_test_enum(TestEnum::Extra);

        root.reborrow().set_test_enum_remote(TestEnum::Val1);

        let mut test_union = root.reborrow().init_test_union();
        test_union.reborrow().set_union_val1(());

        let mut test_union_pure = root.reborrow().init_test_union_pure();
        test_union_pure.reborrow().set_union_val1(());

        assert!(TestStruct::read(builder.get_root_as_reader().unwrap()).is_ok());
    }

    {
        let mut root = builder.init_root();

        root.reborrow().set_test_enum(TestEnum::Val1);

        root.reborrow().set_test_enum_remote(TestEnum::Val1);

        let mut test_union = root.reborrow().init_test_union();
        test_union.reborrow().set_union_val1(());

        let mut test_union_pure = root.reborrow().init_test_union_pure();
        test_union_pure.reborrow().set_union_val1(());

        assert!(TestStruct::read(builder.get_root_as_reader().unwrap()).is_ok());
    }
}

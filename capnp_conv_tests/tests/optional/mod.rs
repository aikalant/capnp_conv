use capnp::message::TypedBuilder;
use capnp_conv::{Readable, Writeable};

use self::test_rust::{BasicStruct, TestDefaults, TestOptional};
use crate::test_capnp::{test_defaults, test_optional};

mod test_rust;

mod optional_test {
    use super::*;
    #[test]
    fn check() {
        assert_identical(&TestOptional {
            prim: Some(5),
            struc: Some(BasicStruct { val: () }),
            text: Some("hello".to_string()),
            data: Some(vec![1, 2, 3, 4]),
            list: Some(vec![5, 6, 7, 8]),
        });
        assert_match(
            &TestOptional {
                prim: None,
                struc: None,
                text: None,
                data: None,
                list: None,
            },
            &TestOptional {
                prim: Some(0),
                struc: None,
                text: None,
                data: None,
                list: None,
            },
        );
    }

    fn assert_identical(input: &TestOptional) {
        assert_match(input, input);
    }

    fn assert_match(input: &TestOptional, expected_output: &TestOptional) {
        let mut builder = TypedBuilder::<test_optional::Owned>::new_default();

        input.write(builder.init_root()).unwrap();
        let reader = builder.get_root_as_reader().unwrap();

        let output = TestOptional::read(reader).unwrap();
        assert!(*expected_output == output)
    }
}

mod defaults_test {
    use super::{test_rust::TestDefaultsOptional, *};
    #[test]
    fn check() {
        let defaults = TestDefaults {
            prim: 999,
            struc: BasicStruct { val: () },
            text: "default".to_string(),
            data: vec![0, 1, 2],
            list: vec![10, 9, 8],
        };

        assert_identical(&TestDefaultsOptional {
            prim: Some(5),
            struc: Some(BasicStruct { val: () }),
            text: Some("hello".to_string()),
            data: Some(vec![1, 2, 3, 4]),
            list: Some(vec![5, 6, 7, 8]),
        });

        assert_match(
            &TestDefaultsOptional {
                prim: None,
                struc: None,
                text: None,
                data: None,
                list: None,
            },
            &TestDefaultsOptional {
                prim: Some(defaults.prim),
                struc: None,
                text: None,
                data: None,
                list: None,
            },
        );

        assert_match_asymmetric(
            &TestDefaultsOptional {
                prim: None,
                struc: None,
                text: None,
                data: None,
                list: None,
            },
            &defaults,
        );
    }

    fn assert_identical(input: &TestDefaultsOptional) {
        assert_match(input, input);
    }

    fn assert_match(input: &TestDefaultsOptional, expected_output: &TestDefaultsOptional) {
        let mut builder = TypedBuilder::<test_defaults::Owned>::new_default();

        input.write(builder.init_root()).unwrap();
        let reader = builder.get_root_as_reader().unwrap();

        let output = TestDefaultsOptional::read(reader).unwrap();
        assert!(*expected_output == output)
    }

    fn assert_match_asymmetric(input: &TestDefaultsOptional, expected_output: &TestDefaults) {
        let mut builder = TypedBuilder::<test_defaults::Owned>::new_default();

        input.write(builder.init_root()).unwrap();
        let reader = builder.get_root_as_reader().unwrap();

        let output = TestDefaults::read(reader).unwrap();
        assert!(*expected_output == output)
    }
}

#[allow(unused, dead_code, clippy::all)]
pub mod optional_capnp;
mod optional_rust;

use optional_rust::{BasicStruct, TestDefaults, TestDefaultsOptional, TestOptional};

use crate::{assert_identical, assert_match};

#[test]
fn optional_test() {
    assert_identical(&TestOptional {
        prim: Some(5),
        struc: Some(BasicStruct { val: 10 }),
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
#[test]
fn defaults_test() {
    let defaults = TestDefaults {
        prim: 999,
        struc: BasicStruct { val: 5 },
        text: "default".to_string(),
        data: vec![0, 1, 2],
        list: vec![10, 9, 8],
    };

    assert_identical(&TestDefaultsOptional {
        prim: Some(5),
        struc: Some(BasicStruct { val: 10 }),
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

    assert_match(
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

#[allow(unused, dead_code, clippy::all, clippy::pedantic)]
mod union_variant_capnp;
mod union_variant_rust;

use union_variant_rust::{BasicStruct, UnionStruct};

use self::union_variant_rust::UnionStructPure;
use crate::assert_identical;

#[test]
fn union_variant_test() {
    assert_identical(&UnionStruct {
        prim: 5,
        union_val1: Some("t".to_owned()),
        union_val2: None,
    });
    assert_identical(&UnionStruct {
        prim: 5,
        union_val1: None,
        union_val2: Some(BasicStruct { val: 10 }),
    });
    assert_identical(&UnionStructPure {
        union_val1: Some("t".to_owned()),
        union_val2: None,
    });
    assert_identical(&UnionStructPure {
        union_val1: None,
        union_val2: Some(BasicStruct { val: 10 }),
    });
}

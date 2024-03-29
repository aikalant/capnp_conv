mod common;
mod optional;
mod union_variant;

use std::fmt::Debug;

use capnp::{message::TypedBuilder, traits::Owned};
use capnp_conv::{Readable, Writable};

fn assert_identical<T, CapT>(input: &T)
where
    T: PartialEq + Debug + Writable<OwnedType = CapT> + Readable<OwnedType = CapT>,
    CapT: Owned,
{
    assert_match(input, input);
}

fn assert_match<T, Y, Cap>(input: &T, expected_output: &Y)
where
    T: PartialEq + Debug + Writable<OwnedType = Cap> + Readable<OwnedType = Cap>,
    Y: PartialEq + Debug + Writable<OwnedType = Cap> + Readable<OwnedType = Cap>,
    Cap: Owned,
{
    let mut builder = TypedBuilder::<Cap>::new_default();
    input.write(builder.init_root());

    let output = Y::read(builder.get_root_as_reader().unwrap()).unwrap();

    assert_eq!(output, *expected_output);
}

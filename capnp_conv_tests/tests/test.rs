mod common;
mod optional;

use std::fmt::Debug;

use capnp::{message::TypedBuilder, traits::Owned};
use capnp_conv::{Readable, Writable};

fn assert_identical<T, CapT>(input: &T)
where
    T: PartialEq + Debug,
    T: Writable<OwnedType = CapT>,
    T: Readable<OwnedType = CapT>,
    CapT: for<'c> Owned<'c>,
{
    assert_match(input, input);
}

fn assert_match<T, Y, Cap>(input: &T, expected_output: &Y)
where
    T: PartialEq + Debug,
    T: Writable<OwnedType = Cap>,
    T: Readable<OwnedType = Cap>,
    Y: PartialEq + Debug,
    Y: Writable<OwnedType = Cap>,
    Y: Readable<OwnedType = Cap>,
    Cap: for<'c> Owned<'c>,
{
    let mut builder = TypedBuilder::<Cap>::new_default();
    input.write(builder.init_root());

    let output = Y::read(builder.get_root_as_reader().unwrap()).unwrap();

    assert_eq!(output, *expected_output);
}

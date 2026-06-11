#[allow(unused, clippy::all, clippy::pedantic)]
#[rustfmt::skip]
mod overrides_capnp;
mod overrides_rust;

use capnp::message::TypedBuilder;
use capnp_conv::{Readable, Writable};
use overrides_rust::Overrides;

#[test]
pub fn check() {
    let mut builder = TypedBuilder::<overrides_capnp::overrides::Owned>::new_default();
    assert_eq!(
        Overrides::read(builder.get_root_as_reader().unwrap()).unwrap(),
        Overrides {
            integer: 5,
            data: "hello_world_read".to_string(),
        }
    );

    Overrides {
        integer: 0,
        data: String::new(),
    }
    .write(builder.init_root());

    let reader = builder.get_root_as_reader().unwrap();
    assert_eq!(reader.get_integer(), 10);
    assert_eq!(reader.get_data().unwrap(), b"hello_world_write");
}

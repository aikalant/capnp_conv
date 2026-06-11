use capnp_conv::capnp_conv;

use super::overrides_capnp::overrides;

#[capnp_conv(overrides)]
#[derive(Debug, Clone, PartialEq)]
pub struct Overrides {
  #[capnp_conv(write_with = "write_integer")]
  #[capnp_conv(read_with = "read_integer")]
  pub integer: i16,
  #[capnp_conv(write_with = "write_data")]
  #[capnp_conv(read_with = "read_data")]
  pub data: String,
}

#[allow(clippy::unnecessary_wraps)]
fn read_integer(
  _: <overrides::Owned as capnp::traits::Owned>::Reader<'_>,
) -> Result<i16, capnp::Error> {
  Ok(5)
}

fn write_integer(
  _: &Overrides,
  builder: &mut <overrides::Owned as capnp::traits::Owned>::Builder<'_>,
) {
  builder.set_integer(10);
}

#[allow(clippy::unnecessary_wraps)]
fn read_data(
  _: <overrides::Owned as capnp::traits::Owned>::Reader<'_>,
) -> Result<String, capnp::Error> {
  Ok("hello_world_read".to_string())
}

fn write_data(
  _: &Overrides,
  builder: &mut <overrides::Owned as capnp::traits::Owned>::Builder<'_>,
) {
  builder.set_data(b"hello_world_write");
}

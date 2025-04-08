#![deny(
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::string_slice,
    clippy::pedantic
)]
#![forbid(unsafe_code)]

use capnp::{Result, traits::Owned};
pub use capnp_conv_macros::capnp_conv;

pub trait Writable {
    type OwnedType: Owned;

    fn write(&self, builder: <Self::OwnedType as Owned>::Builder<'_>);
}

pub trait Readable
where
    Self: Sized,
{
    type OwnedType: Owned;

    #[allow(clippy::missing_errors_doc)]
    fn read(reader: <Self::OwnedType as Owned>::Reader<'_>) -> Result<Self>;
}

pub trait RemoteEnum<T> {
    fn to_capnp_enum(&self) -> T;
}

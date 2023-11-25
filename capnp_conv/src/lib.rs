use capnp::{traits::Owned, Result};
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

    fn read(reader: <Self::OwnedType as Owned>::Reader<'_>) -> Result<Self>;
}

pub trait RemoteEnum<T> {
    fn to_capnp_enum(&self) -> T;
}

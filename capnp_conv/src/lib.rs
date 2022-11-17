use capnp::{traits::Owned, Result};
pub use capnp_conv_macros::capnp_conv;

pub trait Writable {
    type OwnedType: for<'c> Owned<'c>;
    fn write(&self, builder: <Self::OwnedType as Owned>::Builder);
}

pub trait Readable
where
    Self: Sized,
{
    type OwnedType: for<'c> Owned<'c>;
    fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self>;
}

pub trait RemoteEnum<T> {
    fn to_capnp_enum(&self) -> T;
}

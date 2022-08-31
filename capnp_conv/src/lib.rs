use capnp::{traits::Owned, Result};
pub use capnp_conv_macros::capnp_conv;

pub trait Writeable {
    type OwnedType: for<'c> Owned<'c>;
    fn write(&self, builder: <Self::OwnedType as Owned>::Builder) -> Result<()>;
}

pub trait Readable
where
    Self: Sized,
{
    type OwnedType: for<'c> Owned<'c>;
    fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self>;
}

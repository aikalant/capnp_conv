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

#[derive(Debug, Default)]
pub struct Error {
    errors: Vec<(String, Box<dyn std::error::Error>)>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((last, rest)) = self.errors.split_last() {
            if !rest.is_empty() {
                write!(f, "{{ ")?;
            }

            for error in rest {
                write!(f, "\"{}\": ", error.0)?;
                error.1.fmt(f)?;
                write!(f, ", ")?;
            }

            write!(f, "\"{}\": ", last.0)?;
            last.1.fmt(f)?;

            if !rest.is_empty() {
                write!(f, " }}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

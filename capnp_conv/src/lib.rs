use capnp::{
    data_list, enum_list, primitive_list, struct_list, text_list,
    traits::{FromU16, Owned, OwnedStruct, ToU16},
    Result,
};
pub use capnp_conv_macros::capnp_conv;
use duplicate::duplicate_item;

pub trait Writable {
    type OwnedType: for<'c> Owned<'c>;
    fn write(&self, builder: <Self::OwnedType as Owned>::Builder);
}

pub trait Readable: Sized {
    type OwnedType: for<'c> Owned<'c>;
    fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self>;
}
pub trait WritableList<T> {
    fn write(&mut self, items: &[T]);
}
pub trait ReadableList<T> {
    fn read(&self) -> Result<Vec<T>>;
}

pub trait RemoteEnum<T> {
    fn to_capnp_enum(&self) -> T;
}
//-----------------------------------------------------------------------------
//---------------------------------Structs-------------------------------------
//-----------------------------------------------------------------------------

impl<T, CapT> WritableList<T> for struct_list::Builder<'_, CapT>
where
    T: Writable<OwnedType = CapT>,
    for<'c> CapT: OwnedStruct<'c> + Owned<'c, Builder = <CapT as OwnedStruct<'c>>::Builder>,
{
    fn write(&mut self, items: &[T]) {
        for (index, item) in items.iter().enumerate() {
            item.write(self.reborrow().get(index as u32));
        }
    }
}

impl<T, CapT> ReadableList<T> for struct_list::Reader<'_, CapT>
where
    T: Readable<OwnedType = CapT>,
    for<'c> CapT: OwnedStruct<'c> + Owned<'c, Reader = <CapT as OwnedStruct<'c>>::Reader>,
{
    fn read(&self) -> Result<Vec<T>> {
        self.iter().map(|reader| T::read(reader)).collect()
    }
}

// pub trait WritableStructList<T: for<'c> OwnedStruct<'c>> {
//     fn write_as_struct_list(&self, builder: struct_list::Builder<T>);
// }
// impl<T, CapT> WritableStructList<CapT> for Vec<T>
// where
//     T: Writable<OwnedType = CapT>,
//     for<'c> CapT: OwnedStruct<'c> + Owned<'c, Builder = <CapT as OwnedStruct<'c>>::Builder>,
// {
//     fn write_as_struct_list(&self, mut builder: struct_list::Builder<CapT>) {
//         for (index, item) in self.iter().enumerate() {
//             item.write(builder.reborrow().get(index as u32));
//         }
//     }
// }

// pub trait ReadableStructList<T: for<'c> OwnedStruct<'c>>: Sized {
//     fn read_as_struct_list(reader: struct_list::Reader<T>) -> Result<Self>;
// }
// impl<T, CapT> ReadableStructList<CapT> for Vec<T>
// where
//     T: Readable<OwnedType = CapT>,
//     for<'c> CapT: OwnedStruct<'c> + Owned<'c, Reader = <CapT as OwnedStruct<'c>>::Reader>,
// {
//     fn read_as_struct_list(reader: struct_list::Reader<CapT>) -> Result<Self> {
//         reader.iter().map(|reader| T::read(reader)).collect()
//     }
// }

// impl<T, CapT> Writable for Vec<T>
// where
//     T: Writable<OwnedType = CapT>,
//     for<'c> CapT: OwnedStruct<'c> + Owned<'c, Builder = <CapT as OwnedStruct<'c>>::Builder>,
// {
//     type OwnedType = struct_list::Owned<CapT>;
//     fn write(&self, mut builder: <Self::OwnedType as Owned>::Builder) {
//         for (index, item) in self.iter().enumerate() {
//             item.write(builder.reborrow().get(index as u32));
//         }
//     }
// }

// impl<T, CapT> Readable for Vec<T>
// where
//     T: Readable<OwnedType = CapT>,
//     for<'c> CapT: OwnedStruct<'c> + Owned<'c, Reader = <CapT as OwnedStruct<'c>>::Reader>,
// {
//     type OwnedType = struct_list::Owned<CapT>;
//     fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self> {
//         reader.iter().map(|reader| T::read(reader)).collect()
//     }
// }

//-----------------------------------------------------------------------------
//---------------------------------Primitives----------------------------------
//-----------------------------------------------------------------------------

#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64])]
impl WritableList<prim_type> for primitive_list::Builder<'_, prim_type> {
    fn write(&mut self, items: &[prim_type]) {
        for (index, item) in items.iter().enumerate() {
            self.set(index as u32, *item);
        }
    }
}
#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64])]
impl ReadableList<prim_type> for primitive_list::Reader<'_, prim_type> {
    fn read(&self) -> Result<Vec<prim_type>> {
        Ok(self.iter().collect())
    }
}
// #[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64])]
// impl Writable for Vec<prim_type> {
//     type OwnedType = primitive_list::Owned<prim_type>;
//     fn write(&self, mut builder: <Self::OwnedType as Owned>::Builder) {
//         builder.write(self);
//     }
// }
// #[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64])]
// impl Readable for Vec<prim_type> {
//     type OwnedType = primitive_list::Owned<prim_type>;
//     fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self> {
//         reader.read()
//     }
// }

//-----------------------------------------------------------------------------
//---------------------------------Text----------------------------------------
//-----------------------------------------------------------------------------

impl WritableList<String> for text_list::Builder<'_> {
    fn write(&mut self, items: &[String]) {
        for (index, item) in items.iter().enumerate() {
            self.set(index as u32, item.as_str());
        }
    }
}
impl ReadableList<String> for text_list::Reader<'_> {
    fn read(&self) -> Result<Vec<String>> {
        self.iter().map(|s| s.map(|s| s.to_owned())).collect()
    }
}
// impl Writable for Vec<String> {
//     type OwnedType = text_list::Owned;
//     fn write(&self, mut builder: <Self::OwnedType as Owned>::Builder) {
//         builder.write(self)
//     }
// }
// impl Readable for Vec<String> {
//     type OwnedType = text_list::Owned;
//     fn read(reader: <Self::OwnedType as Owned>::Reader) -> Result<Self> {
//         reader.read()
//     }
// }

//-----------------------------------------------------------------------------
//---------------------------------Data----------------------------------------
//-----------------------------------------------------------------------------

impl WritableList<Vec<u8>> for data_list::Builder<'_> {
    fn write(&mut self, items: &[Vec<u8>]) {
        for (index, item) in items.iter().enumerate() {
            self.set(index as u32, item);
        }
    }
}
impl ReadableList<Vec<u8>> for data_list::Reader<'_> {
    fn read(&self) -> Result<Vec<Vec<u8>>> {
        self.iter().map(|s| s.map(|s| s.to_owned())).collect()
    }
}

//-----------------------------------------------------------------------------
//---------------------------------Enums---------------------------------------
//-----------------------------------------------------------------------------

impl<T: FromU16 + ToU16> WritableList<T> for enum_list::Builder<'_, T> {
    fn write(&mut self, items: &[T]) {
        for (index, item) in items.iter().enumerate() {
            self.set(index as u32, unsafe { std::ptr::read(item) });
        }
    }
}
impl<T: FromU16 + ToU16> ReadableList<T> for enum_list::Reader<'_, T> {
    fn read(&self) -> Result<Vec<T>> {
        let mut output = Vec::with_capacity(self.len() as usize);
        for item in 0..self.len() {
            output.push(self.get(item)?);
        }
        Ok(output)
    }
}

pub trait WritableRemoteEnumList<T> {
    fn write_remote(&mut self, items: &[T]);
}

impl<T: RemoteEnum<Y>, Y: ToU16 + FromU16> WritableRemoteEnumList<T> for enum_list::Builder<'_, Y> {
    fn write_remote(&mut self, items: &[T]) {
        for (index, item) in items.iter().enumerate() {
            self.set(index as u32, item.to_capnp_enum());
        }
    }
}

pub trait ReadableRemoteEnumList<T> {
    fn read_remote(&self) -> Result<Vec<T>>;
}
impl<T, Y: FromU16 + Into<T> + Copy> ReadableRemoteEnumList<T> for enum_list::Reader<'_, Y> {
    fn read_remote(&self) -> Result<Vec<T>> {
        let mut output = Vec::with_capacity(self.len() as usize);
        for item in 0..self.len() {
            output.push(self.get(item)?.into());
        }
        Ok(output)
    }
}

pub trait WritableEnumList<T> {
    fn write(&self, builder: enum_list::Builder<T>);
}
impl<T: ToU16 + FromU16> WritableEnumList<T> for Vec<T> {
    fn write(&self, mut builder: enum_list::Builder<T>) {
        builder.write(self)
    }
}
pub trait ReadableEnumList<T>: Sized {
    fn read(reader: enum_list::Reader<T>) -> Result<Self>;
}
impl<T: ToU16 + FromU16> ReadableEnumList<T> for Vec<T> {
    fn read(reader: enum_list::Reader<T>) -> Result<Self> {
        reader.read()
    }
}

// pub trait RemoteEnum<T>: From<T> + Into<T> {}
// pub trait WritableRemoteEnumList<T: ToU16 + FromU16> {
//     fn write(&self, builder: enum_list::Builder<T>);
// }
// impl<T: ToU16 + FromU16, Y: RemoteEnum<T>> WritableRemoteEnumList<T> for Vec<Y> {
//     fn write(&self, mut builder: enum_list::Builder<T>) {
//         for (index, item) in self.iter().enumerate() {
//             builder.set(index as u32, unsafe { std::ptr::read(item) }.into());
//         }
//     }
// }
// pub trait ReadableRemoteEnumList<T: ToU16 + FromU16>: Sized {
//     fn read(reader: enum_list::Reader<T>) -> Result<Self>;
// }
// impl<T: ToU16 + FromU16, Y: RemoteEnum<T>> ReadableRemoteEnumList<T> for Vec<Y> {
//     fn read(reader: enum_list::Reader<T>) -> Result<Self> {
//         reader
//             .iter()
//             .map(|s| s.map(Y::from).map_err(capnp::Error::from))
//             .collect()
//     }
// }

//-----------------------------------------------------------------------------
//---------------------------------Lists---------------------------------------
//-----------------------------------------------------------------------------

use std::{
    convert::{TryFrom, TryInto},
    num::TryFromIntError,
};

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

pub trait WritableList<'a, T: 'a> {
    fn write<I: IntoIterator<Item = &'a T>>(&mut self, items: I);
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

impl<'a, T, CapT> WritableList<'a, T> for struct_list::Builder<'_, CapT>
where
    T: 'a + Writable<OwnedType = CapT>,
    for<'c> CapT: OwnedStruct<'c> + Owned<'c, Builder = <CapT as OwnedStruct<'c>>::Builder>,
{
    fn write<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
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

//-----------------------------------------------------------------------------
//---------------------------------Primitives----------------------------------
//-----------------------------------------------------------------------------

#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64];[f32];[f64])]
impl<'a, T: 'a + Copy + Into<prim_type>> WritableList<'a, T>
    for primitive_list::Builder<'_, prim_type>
{
    fn write<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
            self.set(index as u32, (*item).into());
        }
    }
}
#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64];[f32];[f64])]
impl ReadableList<prim_type> for primitive_list::Reader<'_, prim_type> {
    fn read(&self) -> Result<Vec<prim_type>> {
        Ok(self.iter().collect())
    }
}

pub trait WritablePrimitiveList<'a, T: 'a> {
    fn try_write<I: IntoIterator<Item = &'a T>>(
        &mut self,
        items: I,
    ) -> core::result::Result<(), TryFromIntError>;
}
#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64];[f32];[f64])]
impl<'a, T: 'a + Copy + TryInto<prim_type, Error = TryFromIntError>> WritablePrimitiveList<'a, T>
    for primitive_list::Builder<'_, prim_type>
{
    fn try_write<I: IntoIterator<Item = &'a T>>(
        &mut self,
        items: I,
    ) -> core::result::Result<(), TryFromIntError> {
        for (index, item) in items.into_iter().enumerate() {
            self.set(index as u32, (*item).try_into()?);
        }
        Ok(())
    }
}

pub trait ReadablePrimitiveList<T> {
    fn try_read(&self) -> core::result::Result<Vec<T>, TryFromIntError>;
}
#[duplicate_item(prim_type;[bool];[i8];[i16];[i32];[i64];[u8];[u16];[u32];[u64];[f32];[f64])]
impl<T: TryFrom<prim_type, Error = TryFromIntError>> ReadablePrimitiveList<T>
    for primitive_list::Reader<'_, prim_type>
{
    fn try_read(&self) -> core::result::Result<Vec<T>, TryFromIntError> {
        self.iter().map(|item| item.try_into()).collect()
    }
}

//-----------------------------------------------------------------------------
//---------------------------------Text----------------------------------------
//-----------------------------------------------------------------------------

impl<'a, T: 'a + AsRef<str>> WritableList<'a, T> for text_list::Builder<'_> {
    fn write<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
            self.set(index as u32, item.as_ref());
        }
    }
}
impl ReadableList<String> for text_list::Reader<'_> {
    fn read(&self) -> Result<Vec<String>> {
        self.iter().map(|s| s.map(|s| s.to_owned())).collect()
    }
}

//-----------------------------------------------------------------------------
//---------------------------------Data----------------------------------------
//-----------------------------------------------------------------------------

impl<'a> WritableList<'a, Vec<u8>> for data_list::Builder<'_> {
    fn write<I: IntoIterator<Item = &'a Vec<u8>>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
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

impl<'a, T: 'a + FromU16 + ToU16> WritableList<'a, T> for enum_list::Builder<'_, T> {
    fn write<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
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

pub trait WritableRemoteEnumList<'a, T: 'a> {
    fn write_remote<I: IntoIterator<Item = &'a T>>(&mut self, items: I);
}
impl<'a, T: 'a + RemoteEnum<Y>, Y: ToU16 + FromU16> WritableRemoteEnumList<'a, T>
    for enum_list::Builder<'_, Y>
{
    fn write_remote<I: IntoIterator<Item = &'a T>>(&mut self, items: I) {
        for (index, item) in items.into_iter().enumerate() {
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

//-----------------------------------------------------------------------------
//---------------------------------Lists---------------------------------------
//-----------------------------------------------------------------------------

//TODO

use std::{marker::PhantomData, mem::MaybeUninit};

use capnp_conv::capnp_conv;

use super::common_capnp::{self, basic_struct, comprehensive_struct, generic_struct};

#[capnp_conv(comprehensive_struct)]
#[derive(Debug, Clone, PartialEq)]
pub struct ComprehensiveStructBak<T, Y> {
    pub void_val: (),
    pub bool_val: bool,
    pub i8_val: i8,
    pub i16_val: i16,
    pub i32_val: i32,
    pub i64_val: i64,
    pub u8_val: u8,
    pub u16_val: u16,
    pub u32_val: u32,
    pub u64_val: u64,
    pub f32_val: f32,
    pub f64_val: f64,
    pub text_val: String,
    #[capnp_conv(type = "data")]
    pub data_val: Vec<u8>,
    pub u8_list_val: Vec<u8>,
    pub nested_val: BasicStruct,
    pub list_val: Vec<Vec<BasicStruct>>,
    #[capnp_conv(type = "enum")]
    pub enum_val: common_capnp::ComprehensiveStructEnum,
    #[capnp_conv(type = "enum_remote")]
    pub enum_val_remote: ComprehensiveStructEnum,
    #[capnp_conv(type = "group")]
    pub group_val: ComprehensiveStructGroup<T, Y>,
    #[capnp_conv(type = "union")]
    pub union_val: ComprehensiveStructUnion<T, Y>,
    #[capnp_conv(type = "unnamed_union")]
    pub unnamed_union: ComprehensiveStructUnnamedUnion<T, Y>,
    pub t_val: T,
    pub y_val: Y,
    #[capnp_conv(type = "union")]
    pub comprehensive_union: ComprehensiveUnion<T, Y>,
    pub generic_val: GenericStruct<BasicStruct, BasicStruct>,
}

// Recursive expansion of capnp_conv macro
// ========================================

#[derive(Debug, Clone, PartialEq)]
pub struct ComprehensiveStruct<T, Y> {
    pub void_val: (),
    pub bool_val: bool,
    pub i8_val: i8,
    pub i16_val: i16,
    pub i32_val: i32,
    pub i64_val: i64,
    pub u8_val: u8,
    pub u16_val: u16,
    pub u32_val: u32,
    pub u64_val: u64,
    pub f32_val: f32,
    pub f64_val: f64,
    pub text_val: String,
    pub data_val: Vec<u8>,
    pub u8_list_val: Vec<u8>,
    pub nested_val: BasicStruct,
    pub list_val: Vec<Vec<BasicStruct>>,
    pub enum_val: common_capnp::ComprehensiveStructEnum,
    pub enum_val_remote: ComprehensiveStructEnum,
    pub group_val: ComprehensiveStructGroup<T, Y>,
    pub union_val: ComprehensiveStructUnion<T, Y>,
    pub unnamed_union: ComprehensiveStructUnnamedUnion<T, Y>,
    pub t_val: T,
    pub y_val: Y,
    pub comprehensive_union: ComprehensiveUnion<T, Y>,
    pub generic_val: GenericStruct<BasicStruct, BasicStruct>,
}
#[allow(
    clippy::all,
    clippy::pedantic,
    non_camel_case_types,
    unused_variables,
    unused_braces,
    unused_mut
)]
impl<T, __CaPnP__T, Y, __CaPnP__Y> ::capnp_conv::Writable for ComprehensiveStruct<T, Y>
where
    T: ::capnp_conv::Writable<OwnedType = __CaPnP__T>,
    Y: ::capnp_conv::Writable<OwnedType = __CaPnP__Y>,
    __CaPnP__T: for<'c> ::capnp::traits::Owned<'c>,
    __CaPnP__Y: for<'c> ::capnp::traits::Owned<'c>,
{
    type OwnedType = comprehensive_struct::Owned<__CaPnP__T, __CaPnP__Y>;
    fn write(&self, mut builder: <Self::OwnedType as ::capnp::traits::Owned>::Builder) {
        builder.set_void_val(());
        builder.set_bool_val(self.bool_val);
        builder.set_i8_val(self.i8_val);
        builder.set_i16_val(self.i16_val);
        builder.set_i32_val(self.i32_val);
        builder.set_i64_val(self.i64_val);
        builder.set_u8_val(self.u8_val);
        builder.set_u16_val(self.u16_val);
        builder.set_u32_val(self.u32_val);
        builder.set_u64_val(self.u64_val);
        builder.set_f32_val(self.f32_val);
        builder.set_f64_val(self.f64_val);
        builder.set_text_val(&self.text_val);
        builder.set_data_val(&self.data_val);
        {
            let list = &self.u8_list_val;
            let size = list.len();
            let mut builder = builder.reborrow().init_u8_list_val(size as u32);
            for (idx, item) in list.iter().enumerate().take(size) {
                builder.set(idx as u32, *item)
            }
        };
        self.nested_val.write(builder.reborrow().init_nested_val());
        {
            let list = &self.list_val;
            let size = list.len();
            let mut builder = builder.reborrow().init_list_val(size as u32);
            for (idx, item) in list.iter().enumerate().take(size) {
                let list = item;
                let size = list.len();
                let mut builder = builder.reborrow().init(idx as u32, size as u32);
                for (idx, item) in list.iter().enumerate().take(size) {
                    item.write(builder.reborrow().get(idx as u32))
                }
            }
        };
        builder.set_enum_val(self.enum_val);
        builder.set_enum_val_remote(::capnp_conv::RemoteEnum::to_capnp_enum(
            &self.enum_val_remote,
        ));
        self.group_val.write(builder.reborrow().init_group_val());
        self.union_val.write(builder.reborrow().init_union_val());
        self.unnamed_union.write(builder.reborrow());
        self.t_val.write(builder.reborrow().init_t_val());
        self.y_val.write(builder.reborrow().init_y_val());
        self.comprehensive_union
            .write(builder.reborrow().init_comprehensive_union());
        self.generic_val
            .write(builder.reborrow().init_generic_val());
    }
}
#[allow(
    clippy::all,
    clippy::pedantic,
    non_camel_case_types,
    unused_variables,
    unused_braces,
    unused_mut
)]
impl<T, __CaPnP__T, Y, __CaPnP__Y> ::capnp_conv::Readable for ComprehensiveStruct<T, Y>
where
    T: ::capnp_conv::Readable<OwnedType = __CaPnP__T>,
    Y: ::capnp_conv::Readable<OwnedType = __CaPnP__Y>,
    __CaPnP__T: for<'c> ::capnp::traits::Owned<'c>,
    __CaPnP__Y: for<'c> ::capnp::traits::Owned<'c>,
{
    type OwnedType = comprehensive_struct::Owned<__CaPnP__T, __CaPnP__Y>;
    fn read(reader: <Self::OwnedType as ::capnp::traits::Owned>::Reader) -> ::capnp::Result<Self> {
        let mut __this__ = MaybeUninit::<Self>::uninit();
        let __ptr__ = __this__.as_mut_ptr();
        let mut __errors__ = Vec::<(&'static str, ::capnp::Error)>::with_capacity(0);

        unsafe {
            ::std::ptr::addr_of_mut!((*__ptr__).void_val).write(());
            ::std::ptr::addr_of_mut!((*__ptr__).bool_val).write(reader.reborrow().get_bool_val());

            match reader.reborrow().get_text_val() {
                Ok(val) => ::std::ptr::addr_of_mut!((*__ptr__).text_val).write(val.to_owned()),
                Err(err) => __errors__.push(("text_val", err)),
            }

            ::std::ptr::addr_of_mut!((*__ptr__).text_val).drop_in_place()

            //...todo
        }

        if let Some((last, rest)) = __errors__.split_last() {
            let mut description = String::new();
            if !rest.is_empty() {
                description.push_str("{{ ");
            }

            for error in rest {
                description.push('"');
                description.push_str(&error.0);
                description.push_str("\": ");
                description.push_str(&error.1.description);
                description.push_str(", ");
            }

            description.push('"');
            description.push_str(&last.0);
            description.push_str("\": ");
            description.push_str(&last.1.description);

            if !rest.is_empty() {
                description.push_str(" }}");
            }
            Err(::capnp::Error::failed(description))
        } else {
            Ok(unsafe { __this__.assume_init() })
        }

        // Ok(Self {
        //     void_val: (),
        //     bool_val: reader.reborrow().get_bool_val(),
        //     i8_val: reader.reborrow().get_i8_val(),
        //     i16_val: reader.reborrow().get_i16_val(),
        //     i32_val: reader.reborrow().get_i32_val(),
        //     i64_val: reader.reborrow().get_i64_val(),
        //     u8_val: reader.reborrow().get_u8_val(),
        //     u16_val: reader.reborrow().get_u16_val(),
        //     u32_val: reader.reborrow().get_u32_val(),
        //     u64_val: reader.reborrow().get_u64_val(),
        //     f32_val: reader.reborrow().get_f32_val(),
        //     f64_val: reader.reborrow().get_f64_val(),
        //     text_val: reader.reborrow().get_text_val()?.to_owned(),
        //     data_val: reader.reborrow().get_data_val()?.to_owned(),
        //     u8_list_val: {
        //         let reader = reader.reborrow().get_u8_list_val()?;
        //         let size = reader.len();
        //         let mut list = Vec::with_capacity(size as usize);
        //         for idx in 0..size {
        //             list.push(reader.get(idx));
        //         }
        //         list
        //     },
        //     nested_val: BasicStruct::read(reader.reborrow().get_nested_val()?)?,
        //     list_val: {
        //         let reader = reader.reborrow().get_list_val()?;
        //         let size = reader.len();
        //         let mut list = Vec::with_capacity(size as usize);
        //         for idx in 0..size {
        //             list.push({
        //                 let reader = reader.get(idx)?;
        //                 let size = reader.len();
        //                 let mut list = Vec::with_capacity(size as usize);
        //                 for idx in 0..size {
        //                     list.push(BasicStruct::read(reader.get(idx))?);
        //                 }
        //                 list
        //             });
        //         }
        //         list
        //     },
        //     enum_val: reader.reborrow().get_enum_val()?,
        //     enum_val_remote: reader.reborrow().get_enum_val_remote()?.into(),
        //     group_val: ComprehensiveStructGroup::<T, Y>::read(reader.reborrow().get_group_val())?,
        //     union_val: ComprehensiveStructUnion::<T, Y>::read(reader.reborrow().get_union_val())?,
        //     unnamed_union: ComprehensiveStructUnnamedUnion::<T, Y>::read(reader.reborrow())?,
        //     t_val: T::read(reader.reborrow().get_t_val()?)?,
        //     y_val: Y::read(reader.reborrow().get_y_val()?)?,
        //     comprehensive_union: ComprehensiveUnion::<T, Y>::read(
        //         reader.reborrow().get_comprehensive_union(),
        //     )?,
        //     generic_val: GenericStruct::<BasicStruct, BasicStruct>::read(
        //         reader.reborrow().get_generic_val()?,
        //     )?,
        // })
    }
}
#[allow(
    clippy::all,
    clippy::pedantic,
    non_camel_case_types,
    unused_variables,
    unused_braces,
    unused_mut
)]
impl<'a, T, __CaPnP__T, Y, __CaPnP__Y>
    ::std::convert::TryFrom<comprehensive_struct::Reader<'a, __CaPnP__T, __CaPnP__Y>>
    for ComprehensiveStruct<T, Y>
where
    T: ::capnp_conv::Readable<OwnedType = __CaPnP__T>,
    Y: ::capnp_conv::Readable<OwnedType = __CaPnP__Y>,
    __CaPnP__T: for<'c> ::capnp::traits::Owned<'c>,
    __CaPnP__Y: for<'c> ::capnp::traits::Owned<'c>,
{
    type Error = ::capnp::Error;
    fn try_from(
        reader: comprehensive_struct::Reader<'a, __CaPnP__T, __CaPnP__Y>,
    ) -> ::capnp::Result<Self> {
        ::capnp_conv::Readable::read(reader)
    }
}

#[capnp_conv(basic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicStruct {
    pub val: (),
}

#[capnp_conv(comprehensive_struct::nested_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedStruct<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(generic_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericStruct<A, B> {
    pub a_val: A,
    pub b_val: B,
}

#[capnp_conv(common_capnp::ComprehensiveStructEnum)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComprehensiveStructEnum {
    Val1,
    Val2,
}

#[capnp_conv(comprehensive_struct::group_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComprehensiveStructGroup<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(comprehensive_struct::comprehensive_union::group_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComprehensiveUnionGroup<T, Y> {
    pub t_val: T,
    pub y_val: Y,
}

#[capnp_conv(comprehensive_struct::comprehensive_union::union_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveUnionUnion<T, Y> {
    TVal(T),
    YVal(Y),
    #[capnp_conv(name = "GenericVal")]
    StopComplainingClippy(GenericStruct<BasicStruct, BasicStruct>),
}

#[capnp_conv(comprehensive_struct::union_val)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveStructUnion<T, Y> {
    TVal(T),
    YVal(Y),
}

#[capnp_conv(comprehensive_struct)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveStructUnnamedUnion<T, Y> {
    TVal2(T, PhantomData<*const Y>),
    YVal2(()),
}

#[capnp_conv(comprehensive_struct::comprehensive_union)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensiveUnion<T, Y> {
    VoidVal(()),
    BoolVal(bool),
    I8Val(i8),
    TextVal(String),
    #[capnp_conv(type = "data")]
    DataVal(Vec<u8>),
    TVal(T),
    YVal(Y),
    ListVal(Vec<Vec<BasicStruct>>),
    #[capnp_conv(type = "enum")]
    EnumVal(common_capnp::ComprehensiveStructEnum),
    #[capnp_conv(type = "enum_remote")]
    EnumValRemote(ComprehensiveStructEnum),
    NestedVal(BasicStruct),
    #[capnp_conv(type = "group")]
    GroupVal(ComprehensiveUnionGroup<T, Y>),
    #[capnp_conv(type = "union")]
    UnionVal(ComprehensiveUnionUnion<T, Y>),
    GenericVal(GenericStruct<BasicStruct, BasicStruct>),
}

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Path};

use crate::{
    models::{EnumInfo, FieldInfo, FieldType, ItemInfo, StructInfo},
    utils::{as_turbofish, capitalize_first_letter, is_ptr_type, to_capnp_generic, to_ident},
};

impl ItemInfo {
    pub fn generate_impls(&self, capnp_path: &Path) -> TokenStream2 {
        let impls = match self {
            ItemInfo::Struct(struct_info) => vec![
                struct_info.generate_writer_impl(capnp_path),
                struct_info.generate_reader_impl(capnp_path),
                struct_info.generate_try_from_impl(capnp_path),
            ],
            ItemInfo::Enum(enum_info) if enum_info.is_union() => vec![
                enum_info.generate_writer_impl(capnp_path),
                enum_info.generate_reader_impl(capnp_path),
                enum_info.generate_try_from_impl(capnp_path),
            ],
            ItemInfo::Enum(enum_info) => vec![
                enum_info.generate_into_impl(capnp_path),
                enum_info.generate_from_impl(capnp_path),
                enum_info.generate_to_impl(capnp_path),
            ],
        };
        quote! {
          #(
            #[allow(clippy::all, clippy::pedantic, non_camel_case_types, unused_variables, unused_braces, unused_mut)]
            #impls
          )*
        }
    }
}

impl StructInfo {
    fn generate_writer_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let field_writers: Vec<TokenStream2> = self
            .fields
            .iter()
            .map(|field| {
                if field.is_union_field {
                    let rust_field_name = &field.rust_name;
                    let field_writer = field.generate_field_writer_from_reference();
                    quote! {
                      if let Some(val) = &self.#rust_field_name {
                        #field_writer;
                      }
                    }
                } else {
                    field.generate_field_writer_from_owned()
                }
            })
            .collect();

        let write_body = quote! {
          #(#field_writers;)*
        };

        generate_writable_impl(&self.ident, capnp_path, &self.generics, write_body)
    }
    fn generate_reader_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let (union_fields, non_union_fields): (Vec<&FieldInfo>, Vec<&FieldInfo>) =
            self.fields.iter().partition(|f| f.is_union_field);

        let (non_union_field_names, non_union_readers): (Vec<&Ident>, Vec<TokenStream2>) =
            non_union_fields
                .iter()
                .map(|field| {
                    (
                        &field.rust_name,
                        field.generate_field_reader(false, !self.generics.is_empty()),
                    )
                })
                .unzip();

        let match_arms: Vec<TokenStream2> = union_fields
            .iter()
            .map(|field| {
                let variant_name =
                    to_ident(field.get_capnp_name(ToUpperCamelCase::to_upper_camel_case));
                let union_field_readers: Vec<TokenStream2> = union_fields
                    .iter()
                    .map(|union_field| {
                        let field_name = &union_field.rust_name;
                        if &field.rust_name == field_name {
                            let field_reader =
                                union_field.generate_field_reader(true, !self.generics.is_empty());
                            quote!(#field_name: Some(#field_reader))
                        } else {
                            quote!(#field_name: None)
                        }
                    })
                    .collect();

                quote! {
                  #capnp_path::Which::#variant_name(val) => Self {
                    #(#non_union_field_names,)*
                    #(#union_field_readers,)*
                  }
                }
            })
            .collect();

        let reader_body = if match_arms.is_empty() {
            quote! {
              Ok(Self {
                #(#non_union_field_names: #non_union_readers,)*
              })
            }
        } else {
            quote! {
              #(let #non_union_field_names = #non_union_readers;)*
              Ok(match reader.which()? {
                #(#match_arms,)*
              })
            }
        };

        generate_readable_impl(&self.ident, capnp_path, &self.generics, reader_body)
    }
    fn generate_try_from_impl(&self, capnp_path: &Path) -> TokenStream2 {
        generate_try_from_impl(&self.ident, capnp_path, &self.generics)
    }
}

impl EnumInfo {
    fn generate_to_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let rust_name = &self.ident;
        let match_arms = self.fields.iter().map(|field| {
            let rust_field_name = &field.rust_name;
            let capnp_field_name = to_ident(field.get_capnp_name(capitalize_first_letter));

            quote!(#rust_name::#rust_field_name => #capnp_path::#capnp_field_name)
        });
        quote! {
          impl ::capnp_conv::RemoteEnum<#capnp_path> for #rust_name {
            fn to_capnp_enum(&self) -> #capnp_path {
              match self {
                #(#match_arms,)*
              }
            }
          }
        }
    }
    fn generate_into_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let rust_name = &self.ident;
        quote! {
          impl ::core::convert::Into<#capnp_path> for #rust_name {
            fn into(self) -> #capnp_path {
              ::capnp_conv::RemoteEnum::to_capnp_enum(&self)
            }
          }
        }
    }
    fn generate_from_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let rust_name = &self.ident;
        let match_arms = self.fields.iter().map(|field| {
            let rust_field_name = &field.rust_name;
            let capnp_field_name = to_ident(field.get_capnp_name(capitalize_first_letter));

            quote!(#capnp_path::#capnp_field_name => #rust_name::#rust_field_name)
        });
        quote! {
          impl ::core::convert::From<#capnp_path> for #rust_name {
            fn from(other: #capnp_path) -> Self {
              match other {
                #(#match_arms,)*
              }
            }
          }
        }
    }

    fn generate_writer_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let match_arm_writers: Vec<TokenStream2> = self
            .fields
            .iter()
            .map(|field| {
                let rust_variant_name = &field.rust_name;
                let field_writer = field.generate_field_writer_from_reference();
                let tuple_fields = if field.has_phantom_in_variant {
                    quote!(val, _)
                } else {
                    quote!(val)
                };

                quote!(Self::#rust_variant_name(#tuple_fields) => #field_writer)
            })
            .collect();

        let write_body = quote! {
          match self {
            #(#match_arm_writers,)*
          }
        };

        generate_writable_impl(&self.ident, capnp_path, &self.generics, write_body)
    }
    fn generate_reader_impl(&self, capnp_path: &Path) -> TokenStream2 {
        let match_arm_readers: Vec<TokenStream2> = self
      .fields
      .iter()
      .map(|field| {
        let rust_variant_name = &field.rust_name;
        let capnp_field_name = field.get_capnp_name(ToSnakeCase::to_snake_case);
        let capnp_variant_name = to_ident(capnp_field_name.to_upper_camel_case());
        let field_reader = field.generate_field_reader(true, !self.generics.is_empty());
        let variant_fields = if field.has_phantom_in_variant {
          quote!(#field_reader, ::std::marker::PhantomData)
        } else {
          field_reader
        };

        quote! {
          #capnp_path::Which::#capnp_variant_name(val) => Self::#rust_variant_name(#variant_fields)
        }
      })
      .collect();

        let reader_body = quote! {
          Ok(match reader.which()? {
            #(#match_arm_readers,)*
          })
        };

        generate_readable_impl(&self.ident, capnp_path, &self.generics, reader_body)
    }
    fn generate_try_from_impl(&self, capnp_path: &Path) -> TokenStream2 {
        generate_try_from_impl(&self.ident, capnp_path, &self.generics)
    }
    fn is_union(&self) -> bool {
        self.fields
            .iter()
            .any(|f| !matches!(f.field_type, FieldType::EnumVariant))
    }
}

impl FieldInfo {
    fn generate_field_reader(&self, pre_fetched: bool, reborrow_readers: bool) -> TokenStream2 {
        if matches!(self.field_type, FieldType::Phantom) {
            quote!(::std::marker::PhantomData)
        } else if self.skip_read {
            let field_reader = match &self.default_override {
                Some(default_override) => quote!(#default_override()),
                None => self.generate_default_reader(),
            };
            if self.is_optional {
                quote!(Some(#field_reader))
            } else {
                field_reader
            }
        } else {
            let capnp_field_name = self.get_capnp_name(ToSnakeCase::to_snake_case);
            if self.is_optional {
                let field_reader = self.field_type.generate_field_reader(
                    quote!(reader),
                    &capnp_field_name,
                    false,
                    reborrow_readers,
                );
                if is_ptr_type(&self.field_type) {
                    let checker = format_ident!("has_{}", capnp_field_name);
                    quote! {
                      match reader.#checker() {
                        true => Some(#field_reader),
                        false => None,
                      }
                    }
                } else {
                    quote!(Some(#field_reader))
                }
            } else {
                let reader_name = if pre_fetched {
                    quote!(val)
                } else {
                    quote!(reader)
                };
                self.field_type.generate_field_reader(
                    reader_name,
                    &capnp_field_name,
                    pre_fetched,
                    reborrow_readers,
                )
            }
        }
    }

    fn generate_default_reader(&self) -> TokenStream2 {
        let path = match &self.field_type {
            FieldType::Void(_) => return quote!(()),
            FieldType::Primitive(path) => path,
            FieldType::Data(path) => path,
            FieldType::Text(path) => path,
            FieldType::Struct(path) => path,
            FieldType::EnumRemote(path) => path,
            FieldType::Enum(path) => path,
            FieldType::GroupOrUnion(path, _) => path,
            FieldType::UnnamedUnion(path, _) => path,
            FieldType::List(_) => return quote!(Vec::default()),
            FieldType::GenericStruct(path, _) => path,
            _ => unimplemented!(),
        };
        let path = as_turbofish(path);
        quote!(#path::default())
    }

    fn generate_field_writer_from_reference(&self) -> TokenStream2 {
        if self.skip_write || matches!(self.field_type, FieldType::Phantom) {
            quote! {} //noop
        } else {
            let capnp_field_name = self.get_capnp_name(ToSnakeCase::to_snake_case);
            let field_name = quote!(val);
            if self.is_optional {
                let field_writer =
                    self.field_type
                        .generate_field_writer(&field_name, &capnp_field_name, false);
                quote! {
                  if let Some(val) = val {
                    #field_writer;
                  }
                }
            } else {
                self.field_type
                    .generate_field_writer(&field_name, &capnp_field_name, false)
            }
        }
    }
    fn generate_field_writer_from_owned(&self) -> TokenStream2 {
        let rust_field_name = &self.rust_name;
        let capnp_field_name = self.get_capnp_name(ToSnakeCase::to_snake_case);

        if self.skip_write || matches!(self.field_type, FieldType::Phantom) {
            quote! {} //noop
        } else if self.is_optional {
            let field_name = quote!(val);
            let field_writer =
                self.field_type
                    .generate_field_writer(field_name, &capnp_field_name, false);
            quote! {
              if let Some(val) = &self.#rust_field_name {
                #field_writer;
              }
            }
        } else {
            self.field_type.generate_field_writer(
                quote!(self.#rust_field_name),
                &capnp_field_name,
                true,
            )
        }
    }

    fn get_capnp_name(&self, to_case: impl FnOnce(&str) -> String) -> String {
        to_case(
            self.capnp_name_override
                .as_ref()
                .unwrap_or(&self.rust_name)
                .to_string()
                .as_str(),
        )
    }
}

impl FieldType {
    fn generate_field_reader(
        &self,
        reader_name: impl ToTokens,
        capnp_field_name: &str,
        reader_pre_fetched: bool,
        reborrow_readers: bool,
    ) -> TokenStream2 {
        let getter = format_ident!("get_{}", capnp_field_name);
        let reader = if reborrow_readers {
            quote!(#reader_name.reborrow())
        } else {
            quote!(#reader_name)
        };
        let getter = if reader_pre_fetched {
            quote!(#reader_name)
        } else {
            quote!(#reader.#getter())
        };
        match self {
            FieldType::Phantom => unimplemented!(),
            FieldType::EnumVariant => unimplemented!(),
            FieldType::Void(_) => quote!(()),
            FieldType::Primitive(_) => quote!(#getter),
            FieldType::Data(_) => quote!(#getter?.to_owned()),
            FieldType::Text(_) => quote!(#getter?.to_string()?),
            FieldType::Struct(struct_path) => {
                let struct_path = as_turbofish(struct_path);
                quote!(#struct_path::read(#getter?)?)
            }
            FieldType::EnumRemote(_) => quote!(#getter?.into()),
            FieldType::Enum(_) => quote!(#getter?),
            FieldType::GroupOrUnion(path, _) => {
                let path = as_turbofish(path);
                quote!(#path::read(#getter)?)
            }
            FieldType::UnnamedUnion(union_path, _) => {
                let union_path = as_turbofish(union_path);
                quote!(#union_path::read(#reader)?)
            }
            FieldType::List(item_type) => {
                let item_getter = item_type.generate_struct_field_reader_list_item();
                quote! {
                  {
                    let reader = #getter?;
                    let size = reader.len();
                    let mut list = Vec::with_capacity(size as usize);
                    for idx in 0..size {
                      list.push(#item_getter);
                    }
                    list
                  }
                }
            }
            FieldType::GenericStruct(struct_path, _) => {
                let struct_path = as_turbofish(struct_path);
                quote!(#struct_path::read(#getter?)?)
            }
        }
    }

    fn generate_struct_field_reader_list_item(&self) -> TokenStream2 {
        match self {
            FieldType::Void(_) => quote!(()),
            FieldType::Primitive(_) => quote!(reader.get(idx)),
            FieldType::Data(_) => quote!(reader.get(idx)?.to_owned()),
            FieldType::Text(_) => quote!(reader.get(idx)?.to_string()?),
            FieldType::Struct(struct_path) => {
                let struct_path = as_turbofish(struct_path);
                quote!(#struct_path::read(reader.get(idx))?)
            }
            FieldType::EnumRemote(_) => quote!(reader.get(idx)?.into()),
            FieldType::Enum(_) => quote!(reader.get(idx)?),
            FieldType::List(item_type) => {
                let item_getter = item_type.generate_struct_field_reader_list_item();
                quote! {
                  {
                    let reader = reader.get(idx)?;
                    let size = reader.len();
                    let mut list = Vec::with_capacity(size as usize);
                    for idx in 0..size {
                      list.push(#item_getter);
                    }
                    list
                  }
                }
            }
            FieldType::GenericStruct(struct_path, _) => {
                let struct_path = as_turbofish(struct_path);
                quote!(#struct_path::read(reader.get(idx))?)
            }
            _ => unimplemented!(),
        }
    }
    fn generate_field_writer(
        &self,
        field: impl ToTokens,
        capnp_field_name: &str,
        is_owned: bool,
    ) -> TokenStream2 {
        let setter = format_ident!("set_{}", capnp_field_name);
        let initializer = format_ident!("init_{}", capnp_field_name);
        let (deref_field, ref_field) = if is_owned {
            (quote!(#field), quote!(&#field))
        } else {
            (quote!(*#field), quote!(#field))
        };
        match self {
            FieldType::Phantom => unimplemented!(),
            FieldType::EnumVariant => unimplemented!(),
            FieldType::Void(_) => quote!(builder.#setter(())),
            FieldType::Primitive(_) => quote!(builder.#setter(#deref_field)),
            FieldType::Data(_) => quote!(builder.#setter(#ref_field)),
            FieldType::Text(_) => quote!(builder.#setter(#field.as_str())),
            FieldType::Struct(_) => quote!(#field.write(builder.reborrow().#initializer())),
            FieldType::EnumRemote(_) => {
                quote!(builder.#setter(::capnp_conv::RemoteEnum::to_capnp_enum(#ref_field)))
            }
            FieldType::Enum(_) => quote!(builder.#setter(#deref_field)),
            FieldType::GroupOrUnion(_, _) => {
                quote!(#field.write(builder.reborrow().#initializer()))
            }
            FieldType::UnnamedUnion(_, _) => quote!(#field.write(builder.reborrow())),
            FieldType::List(item_type) => {
                let field_setter = item_type.generate_struct_field_writer_list_item();
                quote! {
                  {
                    let list = #ref_field;
                    let size = list.len();
                    let mut builder = builder.reborrow().#initializer(size as u32);
                    for (idx, item) in list.iter().enumerate().take(size) {
                      #field_setter
                    }
                  }
                }
            }
            FieldType::GenericStruct(_, _) => {
                quote!(#field.write(builder.reborrow().#initializer()))
            }
        }
    }
    fn generate_struct_field_writer_list_item(&self) -> TokenStream2 {
        match self {
            FieldType::Void(_) => quote!(builder.set(idx as u32, ())),
            FieldType::Primitive(_) => quote!(builder.set(idx as u32, *item)),
            FieldType::Data(_) => quote!(builder.set(idx as u32, item)),
            FieldType::Text(_) => quote!(builder.set(idx as u32, item)),
            FieldType::Struct(_) => quote!(item.write(builder.reborrow().get(idx as u32))),
            FieldType::EnumRemote(_) => {
                quote!(builder.set(idx as u32, ::capnp_conv::RemoteEnum::to_capnp_enum(item)))
            }
            FieldType::Enum(_) => quote!(builder.set(idx as u32, *item)),
            FieldType::List(item_type) => {
                let field_setter = item_type.generate_struct_field_writer_list_item();
                quote! {
                  let list = item;
                  let size = list.len();
                  let mut builder = builder.reborrow().init(idx as u32, size as u32);
                  for (idx, item) in list.iter().enumerate().take(size) {
                    #field_setter
                  }
                }
            }
            FieldType::GenericStruct(_, _) => {
                quote!(item.write(builder.reborrow().get(idx as u32)))
            }
            _ => unimplemented!(),
        }
    }
}

fn generate_writable_impl(
    rust_name: impl ToTokens,
    capnp_path: impl ToTokens,
    generics: &[Ident],
    func_body: impl ToTokens,
) -> TokenStream2 {
    let capnp_generics: Vec<Ident> = generics.iter().map(to_capnp_generic).collect();
    quote! {
      impl<#(#generics, #capnp_generics),*> ::capnp_conv::Writable for #rust_name<#(#generics),*>
      where
        #(#generics: ::capnp_conv::Writable<OwnedType = #capnp_generics>,)*
        #(#capnp_generics: ::capnp::traits::Owned,)*
      {
        type OwnedType = #capnp_path::Owned<#(#capnp_generics),*>;

        fn write(&self, mut builder: <Self::OwnedType as ::capnp::traits::Owned>::Builder<'_>) {
          #func_body
        }
      }
    }
}

fn generate_readable_impl(
    rust_name: impl ToTokens,
    capnp_path: impl ToTokens,
    generics: &[Ident],
    func_body: impl ToTokens,
) -> TokenStream2 {
    let capnp_generics: Vec<Ident> = generics.iter().map(to_capnp_generic).collect();
    quote! {
      impl<#(#generics, #capnp_generics),*> ::capnp_conv::Readable for #rust_name<#(#generics),*>
      where
        #(#generics: ::capnp_conv::Readable<OwnedType = #capnp_generics>,)*
        #(#capnp_generics: ::capnp::traits::Owned,)*
      {
        type OwnedType = #capnp_path::Owned<#(#capnp_generics),*>;

        fn read(
          reader: <Self::OwnedType as ::capnp::traits::Owned>::Reader<'_>
        ) -> ::capnp::Result<Self> {
          #func_body
        }
      }
    }
}

fn generate_try_from_impl(
    rust_name: impl ToTokens,
    capnp_path: impl ToTokens,
    generics: &[Ident],
) -> TokenStream2 {
    let capnp_generics: Vec<Ident> = generics.iter().map(to_capnp_generic).collect();
    quote! {
      impl<'a, #(#generics, #capnp_generics),*>
      ::std::convert::TryFrom<#capnp_path::Reader<'a, #(#capnp_generics),*>>
      for #rust_name<#(#generics),*>
      where
        #(#generics: ::capnp_conv::Readable<OwnedType = #capnp_generics>,)*
        #(#capnp_generics: ::capnp::traits::Owned,)*
      {
        type Error = ::capnp::Error;

        fn try_from(reader: #capnp_path::Reader<'a, #(#capnp_generics),*>) -> ::capnp::Result<Self> {
          ::capnp_conv::Readable::read(reader)
        }
      }
    }
}

use crate::{Readable, Writable};

impl Writable for String {
    type OwnedType = capnp::text::Owned;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        builder.push_str(self.as_str());
    }
}

impl Readable for String {
    type OwnedType = capnp::text::Owned;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        reader
            .to_string()
            .map_err(|e| capnp::Error::failed(e.to_string()))
    }
}

impl Writable for Vec<u8> {
    type OwnedType = capnp::data::Owned;

    fn write(&self, builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        for (i, byte) in self.iter().take(builder.len()).enumerate() {
            builder[i] = *byte;
        }
    }
}

impl Readable for Vec<u8> {
    type OwnedType = capnp::data::Owned;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.to_vec())
    }
}

impl Writable for Vec<u16> {
    type OwnedType = capnp::primitive_list::Owned<u16>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<u16> {
    type OwnedType = capnp::primitive_list::Owned<u16>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<u32> {
    type OwnedType = capnp::primitive_list::Owned<u32>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<u32> {
    type OwnedType = capnp::primitive_list::Owned<u32>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<u64> {
    type OwnedType = capnp::primitive_list::Owned<u64>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<u64> {
    type OwnedType = capnp::primitive_list::Owned<u64>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<i8> {
    type OwnedType = capnp::primitive_list::Owned<i8>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<i8> {
    type OwnedType = capnp::primitive_list::Owned<i8>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<i16> {
    type OwnedType = capnp::primitive_list::Owned<i16>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<i16> {
    type OwnedType = capnp::primitive_list::Owned<i16>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<i32> {
    type OwnedType = capnp::primitive_list::Owned<i32>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<i32> {
    type OwnedType = capnp::primitive_list::Owned<i32>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<i64> {
    type OwnedType = capnp::primitive_list::Owned<i64>;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, *item);
        }
    }
}

impl Readable for Vec<i64> {
    type OwnedType = capnp::primitive_list::Owned<i64>;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        Ok(reader.iter().collect())
    }
}

impl Writable for Vec<String> {
    type OwnedType = capnp::text_list::Owned;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, item.as_str());
        }
    }
}

impl Readable for Vec<String> {
    type OwnedType = capnp::text_list::Owned;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        reader
            .iter()
            .map(|item| {
                item.and_then(|item| {
                    item.to_string()
                        .map_err(|e| capnp::Error::failed(e.to_string()))
                })
            })
            .collect::<capnp::Result<Vec<String>>>()
    }
}

impl Writable for Vec<Vec<u8>> {
    type OwnedType = capnp::data_list::Owned;

    fn write(&self, mut builder: <Self::OwnedType as capnp::traits::Owned>::Builder<'_>) {
        #[allow(clippy::cast_possible_truncation)]
        for (i, item) in self.iter().take(builder.len() as usize).enumerate() {
            builder.set(i as u32, item);
        }
    }
}

impl Readable for Vec<Vec<u8>> {
    type OwnedType = capnp::data_list::Owned;

    fn read(reader: <Self::OwnedType as capnp::traits::Owned>::Reader<'_>) -> capnp::Result<Self> {
        reader
            .iter()
            .map(|item| item.map(<[u8]>::to_vec))
            .collect::<capnp::Result<Vec<Vec<u8>>>>()
    }
}

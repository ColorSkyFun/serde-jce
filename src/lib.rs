pub mod de;
pub mod error;
pub mod ser;

use std::io::Read;

pub use de::Deserializer;
pub use error::{Error, Result};
pub use ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::de::Value;

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut vec = Vec::with_capacity(128);
    let mut serializer = Serializer::new(&mut vec);
    value.serialize(&mut serializer)?;
    Ok(vec)
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(slice);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub fn from_reader<'a, T, R: Read>(reader: R) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(reader);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub fn from_slice_to_value(slice: &[u8]) -> Result<std::collections::BTreeMap<u8, Value>>
where
{
    let mut deserializer = Deserializer::new(slice);
    deserializer.deserialize_all()
}

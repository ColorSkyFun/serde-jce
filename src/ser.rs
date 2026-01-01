use crate::error::{Error, Result};
use serde::{Serialize, ser};
use std::io::Write;

pub struct Serializer<W> {
    writer: W,
    next_tag: Option<u8>,
    depth: usize,
    index: u8,
}

impl<W: Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer {
            writer,
            next_tag: None,
            depth: 0,
            index: 0,
        }
    }
}

impl<W: Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeMap = Self;

    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<(), Self::Error>;
    type SerializeTupleVariant = ser::Impossible<(), Self::Error>;
    type SerializeStructVariant = ser::Impossible<(), Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if !v {
            self.write_number(0)
        } else {
            self.write_number(1)
        }
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_number(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_number(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        let tag = self.next_tag.take().unwrap_or(0);
        self.write_head(tag, 0x4)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let tag = self.next_tag.take().unwrap_or(0);
        self.write_head(tag, 0x5)?;
        self.writer.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        // 此处将char视作数字
        self.write_number(v as i64)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let len = v.len();
        let tag = self.next_tag.take().unwrap_or(0);
        if len <= 0xFF {
            self.write_head(tag, 0x6)?;
            self.writer.write_all(&[len as u8])?;
            self.writer.write_all(v.as_bytes())?;
            Ok(())
        } else {
            self.write_head(tag, 0x7)?;
            self.writer.write_all(&(len as u32).to_be_bytes())?;
            self.writer.write_all(v.as_bytes())?;
            Ok(())
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let len = v.len();
        let tag = self.next_tag.take().unwrap_or(0);
        self.write_head(tag, 0x0D)?;
        self.writer.write_all(&[0x0])?;

        self.next_tag = Some(0);
        self.write_number(len as i64)?;
        self.writer.write_all(v)?;
        Ok(())
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self> {
        let tag = self.next_tag.take().unwrap_or(0);
        self.write_head(tag, 0x9)?;
        self.next_tag = Some(0);
        self.write_number(len.unwrap_or(0) as i64)?;
        self.index = 0;
        Ok(self)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let tag = self.next_tag.take().unwrap_or(0);
        self.write_head(tag, 0x8)?;
        self.next_tag = Some(0);
        self.write_number(len.unwrap() as i64)?;
        Ok(self)
    }
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.depth += 1;
        if let Some(tag) = self.next_tag {
            self.write_head(tag, 0xA)?
        }
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
    fn serialize_none(self) -> Result<()> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<()> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<()> {
        todo!()
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        todo!()
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<()> {
        todo!()
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<()> {
        todo!()
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()> {
        todo!()
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }
}
impl<W: std::io::Write> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = crate::error::Error;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::error::Result<()>
    where
        T: serde::Serialize + ?Sized,
    {
        let tag = key.parse::<u8>().map_err(|_| {
            crate::error::Error::Message(format!("Field name {} is not a valid JCE tag", key))
        })?;

        self.next_tag = Some(tag);

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.depth -= 1;
        if self.depth != 0 {
            self.writer.write_all(&[0xB])?;
        }
        Ok(())
    }
}

impl<W: std::io::Write> ser::SerializeSeq for &mut Serializer<W> {
    type Error = Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.next_tag = Some(self.index);
        self.index += 1;
        value.serialize(&mut **self)?;
        Ok(())
    }
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: std::io::Write> ser::SerializeTuple for &mut Serializer<W> {
    type Error = Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.next_tag = Some(self.index);
        self.index += 1;
        value.serialize(&mut **self)?;
        Ok(())
    }
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: std::io::Write> ser::SerializeMap for &mut Serializer<W> {
    type Error = Error;
    type Ok = ();

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.next_tag = Some(0);
        key.serialize(&mut **self)?;
        self.next_tag = Some(1);
        value.serialize(&mut **self)?;
        Ok(())
    }
    fn end(self) -> Result<()> {
        Ok(())
    }
    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }
    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }
}

impl<W: std::io::Write> Serializer<W> {
    fn write_head(&mut self, tag: u8, typ: u8) -> std::io::Result<()> {
        if tag < 15 {
            let header = (tag << 4) | typ;
            self.writer.write_all(&[header])?;
        } else {
            let header = (15 << 4) | typ;
            self.writer.write_all(&[header, tag])?;
        }
        Ok(())
    }

    fn write_number(&mut self, v: i64) -> Result<()> {
        let tag = self.next_tag.take().unwrap_or(0);

        match v {
            0 => self.write_head(tag, 12),
            n if n >= i8::MIN as i64 && n <= i8::MAX as i64 => {
                self.write_head(tag, 0)?;
                self.writer.write_all(&(n as i8).to_be_bytes())
            }
            n if n >= i16::MIN as i64 && n <= i16::MAX as i64 => {
                self.write_head(tag, 1)?;
                self.writer.write_all(&(n as i16).to_be_bytes())
            }
            n if n >= i32::MIN as i64 && n <= i32::MAX as i64 => {
                self.write_head(tag, 2)?;
                self.writer.write_all(&(n as i32).to_be_bytes())
            }
            _ => {
                self.write_head(tag, 3)?;
                self.writer.write_all(&v.to_be_bytes())
            }
        }?;
        Ok(())
    }
}

#[test]
fn test_struct() -> Result<()> {
    use std::collections::HashMap;

    use serde;

    #[derive(serde::Serialize)]
    struct Inner {
        #[serde(rename = "1")]
        data1: u32,
        #[serde(rename = "234", with = "serde_bytes")]
        data2: Vec<u8>,
    }

    #[derive(serde::Serialize)]
    struct Outer<K, V> {
        #[serde(rename = "1")]
        data1: u64,
        #[serde(rename = "2")]
        data2: String,
        #[serde(rename = "3")]
        struc: Inner,
        #[serde(rename = "4")]
        list: Vec<u16>,
        #[serde(rename = "5")]
        map: HashMap<K, V>,
    }

    let map = HashMap::from_iter(
        ["1".to_string(), "2".to_string(), "3".to_string()]
            .into_iter()
            .zip(0..3),
    );
    let inner = Inner {
        data1: 0xDEADBEEF,
        data2: vec![0x1, 0x2, 0x3, 0x4],
    };
    let outer = Outer {
        data1: 1234,
        data2: "Test".to_string(),
        struc: inner,
        list: vec![0xFFF, 0xFFE],
        map,
    };

    let serialized = crate::to_vec(&outer)?;
    println!("{:?}", serialized);
    Ok(())
}

#[test]
fn test_literal() -> Result<()> {
    let mut data = std::collections::HashMap::new();
    data.insert("v1", vec![12, 34]);
    let serialized = crate::to_vec(&data)?;
    println!("{:?}", serialized);

    let data = vec![1, 2, 3, 4, 5];
    let serialized = crate::to_vec(&data)?;
    println!("{:?}", serialized);
    Ok(())
}

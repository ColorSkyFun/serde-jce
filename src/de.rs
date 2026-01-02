use crate::error::{Error, Result};
use serde::de;
use serde::de::DeserializeSeed;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum Value {
    Byte(u8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
    Map(Vec<(Value, Value)>),
    Struct(std::collections::BTreeMap<u8, Value>),
    List(Vec<Value>),
    Zero,
}

pub struct Deserializer<R> {
    reader: R,
    peeked_header: Option<(u8, u8)>,
    current_type: Option<u8>,
}

struct TagIdentifier(pub u8);

struct StructAccessor<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R> StructAccessor<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        Self { de }
    }
}

struct SeqAccessor<'a, R> {
    de: &'a mut Deserializer<R>,
    len: usize,
    current: usize,
}

impl<'a, R> SeqAccessor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        Self {
            de,
            len,
            current: 0,
        }
    }
}

struct MapAccessor<'a, R> {
    de: &'a mut Deserializer<R>,
    len: usize,
    current: usize,
}

impl<'a, R> MapAccessor<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        Self {
            de,
            len,
            current: 0,
        }
    }
}

impl<'de, R: Read> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val: u32 = self.get_number()? as u32;
        if val != 0 {
            visitor.visit_bool(true)
        } else {
            visitor.visit_bool(false)
        }
    }
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.get_number()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.get_number()? as i16)
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.get_number()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.get_number()?)
    }
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i128(self.get_number()? as i128)
    }
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.get_number()? as u8)
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.get_number()? as u16)
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.get_number()? as u32)
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.get_number()? as u64)
    }
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u128(self.get_number()? as u128)
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("Missing type".into()))?;
        visitor.visit_f32(match typ {
            4 => self.read_f32()?,
            5 => self.read_f64()? as f32,
            _ => return Err(Error::Message(format!("Invalid int type {}", typ))),
        })
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("Missing type".into()))?;
        visitor.visit_f64(match typ {
            4 => self.read_f32()? as f64,
            5 => self.read_f64()?,
            _ => return Err(Error::Message(format!("Invalid int type {}", typ))),
        })
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_u8(visitor)
    }
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("No type".into()))?;
        let len = match typ {
            6 => self.read_u8()? as usize,
            7 => self.read_u32()? as usize,
            _ => return Err(Error::Message("Not a string type".into())),
        };

        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;

        let s = std::str::from_utf8(&buf).map_err(|_| Error::Message("Invalid UTF-8".into()))?;

        visitor.visit_str(s)
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("Missing type".into()))?;
        if typ != 13 {
            return Err(Error::Message("Expected SimpleList".into()));
        }

        let (_, element_typ) = self.next_header()?;
        if element_typ != 0 {
            return Err(Error::Message(
                "SimpleList must be followed by Type 0".into(),
            ));
        }
        let len = self.get_raw_number()? as usize;
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;

        visitor.visit_byte_buf(buf)
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self.current_type.take();

        if typ != Some(9) {
            return Err(Error::Message("Missign Type".into()));
        }
        let len = self.get_raw_number()? as usize;
        let value = visitor.visit_seq(SeqAccessor::new(self, len))?;
        Ok(value)
    }
    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("Missing type".into()))?;
        if typ != 8 {
            return Err(Error::Message(format!("Expected Map(8), got {}", typ)));
        }

        let len = self.get_raw_number()? as usize;

        visitor.visit_map(MapAccessor::new(self, len))
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self.current_type.take();
        match typ {
            Some(10) => {
                let value = visitor.visit_map(StructAccessor::new(self))?;
                Ok(value)
            }
            None => visitor.visit_map(StructAccessor::new(self)),
            Some(t) => Err(Error::Message(format!("Expected struct (10), found {}", t))),
        }
    }
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let typ = self.current_type.unwrap();
        self.skip_type(typ)?;
        visitor.visit_unit()
    }
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}

impl<'de, 'a, R: Read> serde::de::MapAccess<'de> for StructAccessor<'a, R> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let (tag, typ) = match self.de.next_header() {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };
        if typ == 11 {
            return Ok(None);
        }

        self.de.current_type = Some(typ);

        seed.deserialize(TagIdentifier(tag)).map(Some)
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Self {
        Deserializer {
            reader,
            peeked_header: None,
            current_type: None,
        }
    }

    pub fn deserialize_any_value(&mut self, typ: u8) -> Result<Value> {
        self.current_type = Some(typ);

        match typ {
            0 => Ok(Value::Byte(self.read_u8()?)),
            1 => Ok(Value::Int16(self.read_u16()? as i16)),
            2 => Ok(Value::Int32(self.read_u32()? as i32)),
            3 => Ok(Value::Int64(self.read_u32()? as i64)),
            4 => Ok(Value::Float(self.read_f32()?)),
            5 => Ok(Value::Double(self.read_f64()?)),
            6 | 7 => Ok(Value::String({
                let typ = self
                    .current_type
                    .take()
                    .ok_or(Error::Message("No type".into()))?;
                let len = match typ {
                    6 => self.read_u8()? as usize,
                    7 => self.read_u32()? as usize,
                    _ => return Err(Error::Message("Not a string type".into())),
                };

                let mut buf = vec![0u8; len];
                self.reader.read_exact(&mut buf)?;

                let s = std::str::from_utf8(&buf)
                    .map_err(|_| Error::Message("Invalid UTF-8".into()))?;

                s.into()
            })),
            8 => {
                let len = self.get_raw_number()? as usize;
                let mut map_vec = Vec::with_capacity(len);
                for _ in 0..len {
                    let (_, k_ty) = self.next_header()?;
                    let key = self.deserialize_any_value(k_ty)?;
                    let (_, v_ty) = self.next_header()?;
                    let val = self.deserialize_any_value(v_ty)?;
                    map_vec.push((key, val));
                }
                Ok(Value::Map(map_vec))
            }
            9 => {
                let len = self.get_raw_number()? as usize;
                let mut list = Vec::with_capacity(len);

                for _ in 0..len {
                    let (_, e_ty) = self.next_header()?;
                    let item = self.deserialize_any_value(e_ty)?;
                    list.push(item);
                }
                Ok(Value::List(list))
            }
            10 => {
                let mut fields = std::collections::BTreeMap::new();
                loop {
                    let (t, ty) = self.next_header()?;
                    if ty == 11 {
                        let _ = self.next_header();
                        break;
                    }
                    let val = self.deserialize_any_value(ty)?;
                    fields.insert(t, val);
                }
                Ok(Value::Struct(fields))
            }
            11 => Err(Error::Message("Unexpected Struct End".into())),
            12 => Ok(Value::Zero),
            13 => Ok(Value::Bytes({
                let typ = self
                    .current_type
                    .take()
                    .ok_or(Error::Message("Missing type".into()))?;
                if typ != 13 {
                    return Err(Error::Message("Expected SimpleList".into()));
                }

                let (_, element_typ) = self.next_header()?;
                if element_typ != 0 {
                    return Err(Error::Message(
                        "SimpleList must be followed by Type 0".into(),
                    ));
                }
                let len = self.get_raw_number()? as usize;
                let mut buf = vec![0u8; len];
                self.reader.read_exact(&mut buf)?;
                buf
            })),
            _ => Err(Error::Message(format!("Unkown Type: {}", typ))),
        }
    }

    fn skip_type(&mut self, typ: u8) -> Result<()> {
        match typ {
            0 => {
                self.read_u8()?;
            }
            1 => {
                self.read_u16()?;
            }
            2 => {
                self.read_u32()?;
            }
            3 => {
                self.read_u64()?;
            }
            4 => {
                self.read_f32()?;
            }
            5 => {
                self.read_f64()?;
            }
            6 => {
                let len = self.read_u8()? as u64;
                self.ignore_bytes(len)?;
            }
            7 => {
                let len = self.read_u32()? as u64;
                self.ignore_bytes(len)?;
            }
            8 => {
                let len = self.get_raw_number()?;
                for _ in 0..len * 2 {
                    let (_, t) = self.next_header()?;
                    self.skip_type(t)?;
                }
            }
            9 => {
                let len = self.get_raw_number()?;
                for _ in 0..len {
                    let (_, t) = self.next_header()?;
                    self.skip_type(t)?;
                }
            }
            10 => loop {
                let (_, t) = self.next_header()?;
                if t == 11 {
                    break;
                }
                self.skip_type(t)?;
            },
            11 | 12 => {}
            13 => {
                let _ = self.next_header()?;
                let len = self.get_raw_number()? as u64;
                self.ignore_bytes(len)?;
            }
            _ => return Err(Error::Message(format!("Unknown type to skip: {}", typ))),
        }
        Ok(())
    }

    fn ignore_bytes(&mut self, len: u64) -> Result<()> {
        std::io::copy(&mut self.reader.by_ref().take(len), &mut std::io::sink())?;
        Ok(())
    }

    pub fn deserialize_all(&mut self) -> Result<std::collections::BTreeMap<u8, Value>> {
        let mut root = std::collections::BTreeMap::new();

        loop {
            let header = self.next_header();

            match header {
                Ok((tag, typ)) => {
                    if typ == 11 {
                        break;
                    }
                    let val = self.deserialize_any_value(typ)?;
                    root.insert(tag, val);
                }
                Err(_) => {
                    break;
                }
            }
        }

        Ok(root)
    }

    pub fn next_header(&mut self) -> Result<(u8, u8)> {
        if let Some(header) = self.peeked_header.take() {
            return Ok(header);
        }

        let mut head = [0u8];
        self.reader.read_exact(&mut head).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::Message("EOF ERROR".to_string())
            } else {
                Error::Io(e)
            }
        })?;

        let mut tag = (head[0] & 0xF0) >> 4;
        let typ = head[0] & 0x0F;
        if tag == 15 {
            let mut ext_tag = [0u8; 1];
            self.reader.read_exact(&mut ext_tag)?;
            tag = ext_tag[0];
        }

        Ok((tag, typ))
    }

    pub fn peek_header(&mut self, tag: u8, typ: u8) {
        self.peeked_header = Some((tag, typ));
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;

        Ok(u16::from_be_bytes(buf))
    }
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;

        Ok(u32::from_be_bytes(buf))
    }
    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;

        Ok(u64::from_be_bytes(buf))
    }
    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_exact(&mut buf)?;

        Ok(f32::from_be_bytes(buf))
    }
    fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_exact(&mut buf)?;

        Ok(f64::from_be_bytes(buf))
    }
    /// 读整型，消耗tag
    fn get_raw_number(&mut self) -> Result<i64> {
        let (_tag, typ) = self.next_header()?;
        match typ {
            12 => Ok(0),
            0 => Ok(self.read_u8()? as i64),
            1 => Ok(self.read_u16()? as i64),
            2 => Ok(self.read_u32()? as i64),
            3 => Ok(self.read_u64()? as i64),
            _ => Err(Error::Message(format!("Expected number type, got {}", typ))),
        }
    }

    /// 读整型，不消耗tag
    fn get_number(&mut self) -> Result<i64> {
        let typ = self
            .current_type
            .take()
            .ok_or(Error::Message("Missing type".into()))?;
        Ok(match typ {
            12 => 0,                      // Zero Type
            0 => self.read_u8()? as i64,  // int1
            1 => self.read_u16()? as i64, // int2
            2 => self.read_u32()? as i64, // int4
            3 => self.read_u64()? as i64,
            _ => return Err(Error::Message(format!("Invalid int type {}", typ))),
        })
    }
}

impl<'de> de::Deserializer<'de> for TagIdentifier {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.0.to_string())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum ignored_any
    }
}

impl<'de, 'a, R: Read> de::SeqAccess<'de> for SeqAccessor<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.current >= self.len {
            return Ok(None);
        }

        let (_, typ) = self.de.next_header()?;

        self.de.current_type = Some(typ);

        let value = seed.deserialize(&mut *self.de)?;
        self.current += 1;

        Ok(Some(value))
    }
}

impl<'de, 'a, R: Read> de::MapAccess<'de> for MapAccessor<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.current >= self.len {
            return Ok(None);
        }

        let (_, typ) = self.de.next_header()?;
        self.de.current_type = Some(typ);

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let (_, typ) = self.de.next_header()?;
        self.de.current_type = Some(typ);
        let val = seed.deserialize(&mut *self.de)?;

        self.current += 1;
        Ok(val)
    }
}

#[test]
fn test_struct() -> Result<()> {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize, Debug)]
    struct Booox {
        #[serde(rename = "1")]
        data1: (String, u32),

        #[serde(rename = "3")]
        data2: HashMap<String, String>,
    }
    #[derive(Deserialize, Serialize, Debug)]
    struct Boox {
        #[serde(rename = "1")]
        data1: Option<u32>,
        #[serde(rename = "2")]
        data2: Booox,
    }

    let mut map = HashMap::new();
    map.insert("123".into(), "asd".into());
    let booox = Booox {
        data1: ("hahaha".to_string(), 0),
        data2: map,
    };
    let book = Boox {
        data1: Some(123),
        data2: booox,
    };
    let serialized = crate::to_vec(&book)?;
    println!("{:?}", serialized);

    let boox = crate::from_slice::<Boox>(&serialized);
    println!("{:?}", boox);
    println!("{:?}", crate::from_slice_to_value(&serialized));
    Ok(())
}

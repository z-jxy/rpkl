#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
use serde::de::{self, Visitor};
#[cfg(not(feature = "indexmap"))]
use std::collections::HashMap;
use std::fmt;

use crate::Value;

#[cfg(feature = "trace")]
use tracing::debug;

pub struct PklVisitor;

impl<'de> Visitor<'de> for PklVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(feature = "trace")]
        {
            debug!("PklVisitor failed");
        }
        formatter.write_str("a valid pkl value")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(i64::from(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(i64::from(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        #[cfg(feature = "trace")]
        debug!("visiting i64: {}", value);

        if i32::try_from(value).is_ok() {
            if value >= 0 {
                Ok(Value::Int(crate::internal::Integer::Pos(value as u64)))
            } else {
                Ok(Value::Int(crate::internal::Integer::Neg(value)))
            }
        } else {
            Err(E::custom(format!("i32 out of range: {value}")))
        }
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Boolean(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_i64(i64::from(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(u64::from(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(u64::from(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_u64(u64::from(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        #[cfg(feature = "trace")]
        debug!("visit_u64: {}", v);
        if i64::try_from(v).is_ok() {
            Ok(Value::Int(crate::internal::Integer::Pos(v)))
        } else {
            Err(E::custom(format!("u64 out of range: {v}")))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_f64(f64::from(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Int(crate::internal::Integer::Float(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(de::Error::invalid_type(de::Unexpected::Bytes(v), &self))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_bytes(&v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let _ = deserializer;
        Err(de::Error::invalid_type(de::Unexpected::Option, &self))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(de::Error::invalid_type(de::Unexpected::Unit, &self))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let _ = deserializer;
        Err(de::Error::invalid_type(
            de::Unexpected::NewtypeStruct,
            &self,
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vec = match seq.size_hint() {
            Some(size) => Vec::with_capacity(size),
            None => Vec::new(),
        };

        while let Some(value) = seq.next_element()? {
            vec.push(value);
        }

        Ok(Value::List(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut map_impl = if let Some(size) = map.size_hint() {
            #[cfg(feature = "indexmap")]
            let m = IndexMap::with_capacity(size);
            #[cfg(not(feature = "indexmap"))]
            let m = HashMap::with_capacity(size);
            m
        } else {
            #[cfg(feature = "indexmap")]
            let m = IndexMap::new();
            #[cfg(not(feature = "indexmap"))]
            let m = HashMap::new();
            m
        };
        while let Some((key, value)) = map.next_entry()? {
            map_impl.insert(key, value);
        }
        Ok(Value::Map(map_impl))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let _ = data;
        Err(de::Error::invalid_type(de::Unexpected::Enum, &self))
    }
}

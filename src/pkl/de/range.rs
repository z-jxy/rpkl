use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};

use super::KeyDeserializer;

pub struct RangeDeserializer<'a> {
    pub start: &'a i64,
    pub end: &'a i64,
}

impl<'a, 'de> Deserializer<'de> for RangeDeserializer<'a> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 u8 u16 u32 f32 char string str
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map enum struct identifier ignored_any

        i64 u64 f64
    }

    fn deserialize_any<V>(self, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor
            .visit_map(RangeMapAccess {
                // input: self.input,
                start: self.start,
                end: self.end,
                state: 0,
            })
            .map_err(|e| crate::Error::Message(format!("failed to deserialize range: {}", e)))
    }
}

pub struct RangeMapAccess<'a> {
    // input: &'a str,
    pub state: u8,
    pub start: &'a i64,
    pub end: &'a i64,
}

impl<'a, 'de> MapAccess<'de> for RangeMapAccess<'a> {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.state {
            0 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("start")).map(Some)
            }
            1 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("end")).map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.state {
            // start of range
            1 => seed.deserialize(de::value::I64Deserializer::new(*self.start)),
            // end of range
            2 => seed.deserialize(de::value::I64Deserializer::new(*self.end)),
            _ => Err(de::Error::custom("unexpected state")),
        }
    }
}

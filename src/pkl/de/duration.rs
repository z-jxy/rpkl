use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};

use super::KeyDeserializer;

pub struct DurationDeserializer<'a> {
    pub duration: &'a std::time::Duration,
}

impl<'a, 'de> Deserializer<'de> for DurationDeserializer<'a> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 u8 u16 u32 f32 char string str
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map enum struct identifier ignored_any

        i64 u64 f64
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor
            .visit_map(DurationMapAccess {
                duration: &self.duration,
                state: 0,
            })
            .map_err(|_| crate::Error::Message("failed to deserialize duration".to_string()))
    }
}

struct DurationMapAccess<'a> {
    duration: &'a std::time::Duration,
    state: u8,
}

impl<'a, 'de> MapAccess<'de> for DurationMapAccess<'a> {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.state {
            0 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("secs")).map(Some)
            }
            1 => {
                self.state += 1;
                seed.deserialize(KeyDeserializer("nanos")).map(Some)
            }
            _ => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.state {
            1 => seed.deserialize(de::value::U64Deserializer::new(self.duration.as_secs())),
            2 => seed.deserialize(de::value::U32Deserializer::new(
                self.duration.subsec_nanos(),
            )),
            _ => Err(de::Error::custom("unexpected state")),
        }
    }
}

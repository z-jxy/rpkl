use serde::de::{self, MapAccess};

use super::KeyDeserializer;

pub struct RangeMapAccess<'a> {
    pub state: u8,
    pub start: &'a i64,
    pub end: &'a i64,
}

impl<'de> MapAccess<'de> for RangeMapAccess<'_> {
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

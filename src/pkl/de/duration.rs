use serde::de::{self, MapAccess};

use super::KeyDeserializer;

pub struct DurationMapAccess<'a> {
    pub duration: &'a std::time::Duration,
    pub state: u8,
}

impl<'de> MapAccess<'de> for DurationMapAccess<'_> {
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

use serde::{
    de::{self, Deserializer, MapAccess, Visitor},
    forward_to_deserialize_any,
};

pub struct DurationDeserializer {
    pub input: String,
}

impl<'de> Deserializer<'de> for DurationDeserializer {
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
                input: self.input,
                state: 0,
            })
            .map_err(|_| crate::Error::Message("failed to deserialize duration".to_string()))
    }
}

struct DurationMapAccess {
    input: String,
    state: u8,
}

// TODO: there must be a better way to implement thisß

impl<'de> MapAccess<'de> for DurationMapAccess {
    type Error = de::value::Error;

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
        // TODO: refactor so we dont do this twice
        let (value, unit) = parse_duration(&self.input)?;
        match self.state {
            1 => {
                let secs = match unit {
                    "ns" => 0,
                    "us" => 0,
                    "ms" => 0,
                    "s" => value,
                    "min" => value * 60,
                    "h" => value * 3600,
                    "d" => value * 86400,
                    _ => return Err(de::Error::custom("invalid duration unit")),
                };
                seed.deserialize(de::value::U64Deserializer::new(secs))
            }
            2 => {
                let nanos = match unit {
                    "ns" => value,
                    "us" => value * 1000,
                    "ms" => value * 1_000_000,
                    "s" => 0,
                    "min" => 0,
                    "h" => 0,
                    "d" => 0,
                    _ => return Err(de::Error::custom("invalid duration unit")),
                };
                seed.deserialize(de::value::U32Deserializer::new(nanos as u32))
            }
            _ => Err(de::Error::custom("unexpected state")),
        }
    }
}

fn parse_duration(input: &str) -> Result<(u64, &str), de::value::Error> {
    const UNITS: [&str; 7] = ["ns", "us", "ms", "s", "min", "h", "d"];
    for unit in UNITS.iter() {
        if let Some(value_str) = input.strip_suffix(unit) {
            if let Ok(value) = value_str.parse::<u64>() {
                return Ok((value, unit));
            }
        }
    }
    Err(de::Error::custom("invalid duration format"))
}

struct KeyDeserializer(&'static str);

impl<'de> Deserializer<'de> for KeyDeserializer {
    type Error = de::value::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 u8 u16 u32 f32 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map enum struct identifier ignored_any

        i64 u64 f64
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    // Implement other methods or use forward_to_deserialize_any! for the rest
}

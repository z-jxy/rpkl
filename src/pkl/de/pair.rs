use serde::{
    de::{self, Deserializer, SeqAccess, Visitor},
    forward_to_deserialize_any, Deserialize,
};

use crate::{pkl::deserializer::PklValueDeserializer, Value};

pub struct TupleDeserializer<'a> {
    // pub input: &'a str,
    pub pair: (&'a Value, &'a Value),
}

impl<'a, 'de> Deserializer<'de> for TupleDeserializer<'a> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple_struct map enum struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(2, visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor
            .visit_seq(TupleSeqAccess {
                index: 0,
                pair: (&self.pair.0, &self.pair.1),
            })
            .map_err(|_| crate::Error::Message("failed to deserialize tuple".to_string()))
    }
}

struct TupleSeqAccess<'a> {
    pair: (&'a Value, &'a Value),
    index: usize,
}

impl<'a, 'de> SeqAccess<'de> for TupleSeqAccess<'a> {
    type Error = crate::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= 2 {
            return Ok(None);
        }
        let element = match self.index {
            0 => self.pair.0,
            1 => self.pair.1,
            _ => unreachable!(),
        };

        self.index += 1;

        seed.deserialize(PklValueDeserializer(&element)).map(Some)
    }
}

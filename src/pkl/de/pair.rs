use serde::de::{self, SeqAccess};

use crate::{Value, pkl::deserializer::PklValueDeserializer};

pub struct TupleSeqAccess<'a> {
    pub pair: (&'a Value, &'a Value),
    pub index: usize,
}

impl<'de> SeqAccess<'de> for TupleSeqAccess<'_> {
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

        seed.deserialize(PklValueDeserializer(element)).map(Some)
    }
}

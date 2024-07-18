use serde::{
    de::{Deserializer, Visitor},
    forward_to_deserialize_any,
};

/// Deserializer for struct keys
pub(crate) struct KeyDeserializer(pub(crate) &'static str);
impl<'de> Deserializer<'de> for KeyDeserializer {
    type Error = crate::Error;

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
}

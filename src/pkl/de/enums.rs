use serde::de::{self, DeserializeSeed, EnumAccess, VariantAccess, Visitor};

use crate::pkl::deserializer::PklValueDeserializer;

pub struct EnumDeserializer<'a> {
    de: PklValueDeserializer<'a>,
}

impl<'a> EnumDeserializer<'a> {
    pub fn new(de: PklValueDeserializer<'a>) -> Self {
        EnumDeserializer { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for EnumDeserializer<'a> {
    type Error = crate::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> crate::Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(self.de)?;

        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for EnumDeserializer<'a> {
    type Error = crate::Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> crate::Result<()> {
        Err(crate::Error::Message(
            "unit_variant :: ExpectedString".into(),
        ))
    }

    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> crate::Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> crate::Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

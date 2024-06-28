use std::collections::HashMap;

use crate::pkl::internal::{self};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

use crate::pkl::internal::PklValue;
use crate::pkl::{self};

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    map: &'de HashMap<String, PklValue>,
}

impl<'de> Deserializer<'de> {
    pub fn from_pkl_map(map: &'de HashMap<String, PklValue>) -> Self {
        Deserializer { map }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        trace!("deserialize_any, using MapAccessImpl map");
        visitor.visit_map(MapAccessImpl::new(self))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        let _span = span!(Level::INFO, "deserialize_enum").entered();
        #[cfg(feature = "trace")]
        debug!("deserialize_enum: {:?}, {:?}", _name, _variants);

        let ret = visitor.visit_enum(Enum::new(self));

        #[cfg(feature = "trace")]
        _span.exit();

        ret
    }
}

struct SeqAccessDeserializer<'a, 'de: 'a> {
    seq: SeqAccessImpl<'a, 'de>,
}

impl<'de, 'a> de::Deserializer<'de> for SeqAccessDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("deserialize_enum seq");

        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
}

struct MapAccessImpl<'a, 'de: 'a> {
    // key: &'a str,
    de: &'a mut Deserializer<'de>,
    index: usize,
    keys: Vec<&'de String>,
}

impl<'a, 'de> MapAccessImpl<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        let keys = de.map.keys().collect();
        MapAccessImpl { de, keys, index: 0 }
    }
}

impl<'de, 'a> MapAccess<'de> for MapAccessImpl<'a, 'de> {
    type Error = Error;
    // type Value = PklValue;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("tracing next_key_seed");

        if self.index < self.keys.len() {
            let key = self.keys[self.index];
            self.index += 1;
            #[cfg(feature = "trace")]
            debug!("looking up key: {:?}", key);
            seed.deserialize(key.as_str().into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("next_value_seed");
        let key = self.keys[self.index - 1];
        if let Some(value) = self.de.map.get(key) {
            #[cfg(feature = "trace")]
            debug!("found value for key: {:?}: {:?}", key, value);
            match value {
                PklValue::Int(i) => match i {
                    internal::Integer::Pos(u) => seed.deserialize((*u).into_deserializer()),
                    internal::Integer::Neg(n) => seed.deserialize((*n).into_deserializer()),
                    internal::Integer::Float(f) => seed.deserialize((*f).into_deserializer()),
                },
                PklValue::String(s) => seed.deserialize(s.as_str().into_deserializer()),

                PklValue::List(elements) => {
                    #[cfg(feature = "trace")]
                    let _span = span!(Level::INFO, "start parsing list").entered();

                    #[cfg(feature = "trace")]
                    debug!("parsing list: {:?}", elements);

                    let seq = SeqAccessImpl::new(self.de, elements);
                    let result = seed.deserialize(SeqAccessDeserializer { seq });

                    #[cfg(feature = "trace")]
                    _span.exit();

                    result
                }
                PklValue::Map(m) => seed.deserialize(&mut Deserializer::from_pkl_map(m)),
                PklValue::Boolean(b) => seed.deserialize(b.into_deserializer()),
                PklValue::Null => seed.deserialize(().into_deserializer()),
            }
        } else {
            Err(Error::Message(format!("no value found for: {key}")))
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////
struct SeqAccessImpl<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    elements: &'de [PklValue],
    index: usize,
}

impl<'a, 'de> SeqAccessImpl<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, elements: &'de [PklValue]) -> Self {
        SeqAccessImpl {
            de,
            elements,
            index: 0,
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for SeqAccessImpl<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("next_element_seed");

        if self.index < self.elements.len() {
            let value = &self.elements[self.index];
            self.index += 1;
            #[cfg(feature = "trace")]
            debug!("el value: {:?}", value);
            let ret = match value {
                PklValue::Int(i) => match i {
                    pkl::internal::Integer::Pos(u) => {
                        seed.deserialize((*u).into_deserializer()).map(Some)
                    }
                    pkl::internal::Integer::Float(f) => {
                        seed.deserialize((*f).into_deserializer()).map(Some)
                    }
                    pkl::internal::Integer::Neg(n) => {
                        seed.deserialize((*n).into_deserializer()).map(Some)
                    }
                },
                PklValue::String(s) => seed.deserialize(s.as_str().into_deserializer()).map(Some),
                PklValue::List(elements) => seed
                    .deserialize(SeqAccessDeserializer {
                        seq: SeqAccessImpl::new(self.de, elements),
                    })
                    .map(Some),
                PklValue::Map(m) => seed
                    .deserialize(&mut Deserializer::from_pkl_map(m))
                    .map(Some),
                PklValue::Boolean(b) => seed.deserialize(b.into_deserializer()).map(Some),
                PklValue::Null => seed.deserialize(().into_deserializer()).map(Some),
            };

            return ret;
        } else {
            #[cfg(feature = "trace")]
            debug!("no more elements");
            Ok(None)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("EnumAccess variant_seed");

        let val = match seed.deserialize(&mut *self.de) {
            Ok(v) => v,
            Err(e) => {
                #[cfg(feature = "trace")]
                error!("Failed to deserialize variant seed: {:?}", e);
                return Err(e);
            }
        };
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        debug!("struct_variant, : {:?}", _fields);

        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "dhat-heap")]
    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    #[test]
    fn deserialize() {
        use super::*;
        use pkl::PklSerialize;
        use rmpv::Value;
        use serde::Deserialize;

        #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();

        #[derive(Debug, PartialEq, Deserialize)]
        struct Config {
            ip: String,
            port: u16,
            birds: Vec<String>,
            database: Database,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct Database {
            username: String,
            password: String,
        }

        let ast = Value::Array(vec![
            Value::Integer(1.into()),
            Value::String("example".into()),
            Value::String("file:///Users/testing/rpkl/examples/example.pkl".into()),
            Value::Array(vec![
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("ip".into()),
                    Value::String("127.0.0.1".into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("port".into()),
                    Value::Integer(8080.into()),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("birds".into()),
                    Value::Array(vec![
                        Value::Integer(5.into()),
                        Value::Array(vec![
                            Value::String("Pigeon".into()),
                            Value::String("Hawk".into()),
                            Value::String("Penguin".into()),
                        ]),
                    ]),
                ]),
                Value::Array(vec![
                    Value::Integer(16.into()),
                    Value::String("database".into()),
                    Value::Array(vec![
                        Value::Integer(1.into()),
                        Value::String("Dynamic".into()),
                        Value::String("pkl:base".into()),
                        Value::Array(vec![
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("username".into()),
                                Value::String("admin".into()),
                            ]),
                            Value::Array(vec![
                                Value::Integer(16.into()),
                                Value::String("password".into()),
                                Value::String("secret".into()),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]);

        let pkl_mod =
            crate::api::parser::pkl_eval_module(&ast).expect("failed to evaluate pkl ast");
        let mut mapped = pkl_mod
            .serialize_pkl_ast()
            .expect("failed to serialize pkl module");

        let deserialized = Config::deserialize(&mut Deserializer::from_pkl_map(&mut mapped))
            .expect("failed to deserialize");

        let expected = Config {
            ip: "127.0.0.1".into(),
            port: 8080,
            birds: vec!["Pigeon".into(), "Hawk".into(), "Penguin".into()],
            database: Database {
                username: "admin".to_owned(),
                password: "secret".to_owned(),
            },
        };
        assert_eq!(expected, deserialized)
    }
}

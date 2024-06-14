use std::collections::BTreeMap;

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

use crate::pkl::{self, PklValue};

use super::error::{Error, Result};

pub struct Deserializer<'de> {
    map: &'de std::collections::BTreeMap<String, PklValue>,
}

impl<'de> Deserializer<'de> {
    // By convention, `Deserializer` constructors are named like `from_xyz`.
    pub fn from_pkl_map(map: &'de BTreeMap<String, PklValue>) -> Self {
        Deserializer { map }
    }
}

pub fn from_pkl_map<'a, T>(map: &'a BTreeMap<String, PklValue>) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(&mut Deserializer::from_pkl_map(&map))
}

// impl<'de> Deserializer<'de> {}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        trace!("deserialize_any");
        visitor.visit_map(MapAccessImpl::new(self))
    }

    // fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    // where
    //     V: Visitor<'de>,
    // {
    //     #[cfg(feature = "trace")]
    //     trace!("deserialize_seq");
    //     // TODO: lifetimes are 'v annoying, refactor to use them properly.
    //     let values = self.map.values().map(|v| v.to_owned()).collect::<Vec<_>>();
    //     let seq = SeqAccessImpl::new(self, values.as_slice());
    //     visitor.visit_seq(seq)
    // }
}

struct SeqAccessDeserializer<'a, 'de: 'a> {
    seq: SeqAccessImpl<'a, 'de>,
}

impl<'de, 'a> de::Deserializer<'de> for SeqAccessDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
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
        debug!("tracing next_value_seed");
        let key = self.keys[self.index - 1];
        if let Some(value) = self.de.map.get(key) {
            match value {
                // PklValue::Int(i) => seed.deserialize(i.into_deserializer()),
                PklValue::Int(i) => match i {
                    pkl::Integer::Pos(u) => seed.deserialize((*u).into_deserializer()),
                    pkl::Integer::Neg(n) => seed.deserialize((*n).into_deserializer()),
                    pkl::Integer::Float(f) => seed.deserialize((*f).into_deserializer()),
                },

                PklValue::String(s) => seed.deserialize(s.as_str().into_deserializer()),
                // PklValue::List(a) => {
                //     seed.deserialize(Deserializer::from_pkl_map(a).into_deserializer())
                // }
                PklValue::List(elements) => {
                    #[cfg(feature = "trace")]
                    let _span = span!(Level::INFO, "start parsing list").entered();

                    // TODO: figure out lifetimes for this
                    // let el = elements.iter().map(|v| v).collect::<Vec<_>>();
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
            Err(Error::Message("value missing".into()))
        }
    }
}

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
        if self.index < self.elements.len() {
            let value = &self.elements[self.index];
            self.index += 1;
            match value {
                PklValue::Int(i) => match i {
                    pkl::Integer::Pos(u) => seed.deserialize((*u).into_deserializer()).map(Some),
                    pkl::Integer::Float(f) => seed.deserialize((*f).into_deserializer()).map(Some),
                    pkl::Integer::Neg(n) => seed.deserialize((*n).into_deserializer()).map(Some),
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
            }
        } else {
            Ok(None)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/*

*/

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
            Value::String("file:///Users/testing/code/rust/pkl-rs/examples/example.pkl".into()),
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

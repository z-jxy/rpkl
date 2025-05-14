use crate::pkl::de::{DurationMapAccess, EnumDeserializer, RangeMapAccess, TupleSeqAccess};
use crate::value::datasize::DataSizeMapAccess;
use crate::value::value::MapImpl;
use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

use crate::Value as PklValue;

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    map: &'de MapImpl<String, PklValue>,
}

impl<'de> Deserializer<'de> {
    pub fn from_pkl_map(map: &'de MapImpl<String, PklValue>) -> Self {
        Deserializer { map }
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str option string
        bytes byte_buf unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        trace!("deserialize_any, using MapAccessImpl map");
        visitor.visit_map(MapAccessImpl::new(self))
    }
}

////////////////////////////////////////////////////////////////////////////////
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

impl<'de> MapAccess<'de> for MapAccessImpl<'_, 'de> {
    type Error = Error;

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
        debug!("[next_value_seed]");

        let key = self.keys[self.index - 1];

        let Some(value) = self.de.map.get(key) else {
            return Err(Error::Message(format!("no value found for: {key}")));
        };

        #[cfg(feature = "trace")]
        debug!("next_value_seed for key: {:?}: {:?}", key, value);
        seed.deserialize(value.into_deserializer())
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct PklSeqAccess<'a> {
    elements: std::slice::Iter<'a, PklValue>,
}

impl<'a> PklSeqAccess<'a> {
    fn new(elements: &'a [PklValue]) -> Self {
        PklSeqAccess {
            elements: elements.iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for PklSeqAccess<'_> {
    type Error = crate::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.elements.next() {
            Some(element) => {
                let deserializer = element.into_deserializer();
                seed.deserialize(deserializer).map(Some)
            }
            None => Ok(None),
        }
    }
}
////////////////////////////////////////////////////////////////////////////////

struct PklMapAccess<'a> {
    // key: &'a str,
    de: &'a mut Deserializer<'a>,
    index: usize,
    keys: Vec<&'a String>,
}

impl<'a> PklMapAccess<'a> {
    fn new(de: &'a mut Deserializer<'a>) -> Self {
        let keys = de.map.keys().collect();
        PklMapAccess { de, keys, index: 0 }
    }
}

impl<'de> MapAccess<'de> for PklMapAccess<'_> {
    type Error = Error;

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
        debug!("[next_value_seed]");

        let key = self.keys[self.index - 1];

        let Some(value) = self.de.map.get(key) else {
            return Err(Error::Message(format!("no value found for: {key}")));
        };

        #[cfg(feature = "trace")]
        debug!("next_value_seed for key: {:?}: {:?}", key, value);
        seed.deserialize(value.into_deserializer())
    }
}

/// Internal deserializer used for deserializing Tuples from `PklValue`
#[derive(Clone, Copy)]
pub struct PklValueDeserializer<'v>(pub &'v PklValue);

impl<'de> serde::Deserializer<'de> for PklValueDeserializer<'_> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
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
        // unit variants are handled upfront
        if let Some(str_val) = self.0.as_str() {
            visitor.visit_enum(str_val.into_deserializer())
        } else {
            visitor.visit_enum(EnumDeserializer::new(self))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.0 == &PklValue::Null {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "trace")]
        trace!("[PklValueDeserializer] deserialize_any : {:?}", self.0);

        match self.0 {
            PklValue::Int(i) => match i {
                crate::internal::Integer::Pos(u) => visitor.visit_u64(*u),
                crate::internal::Integer::Neg(n) => visitor.visit_i64(*n),
                crate::internal::Integer::Float(f) => visitor.visit_f64(*f),
            },
            PklValue::String(s) | PklValue::Regex(s) => visitor.visit_string(s.to_owned()),

            PklValue::Boolean(b) => visitor.visit_bool(*b),
            PklValue::Null => visitor.visit_unit(),

            PklValue::List(elements) => visitor.visit_seq(PklSeqAccess::new(elements)),

            PklValue::Range(r) => visitor.visit_map(RangeMapAccess {
                start: &r.start,
                end: &r.end,
                state: 0,
            }),

            PklValue::Duration(duration) => {
                visitor.visit_map(DurationMapAccess { duration, state: 0 })
            }
            PklValue::Pair(a, b) => visitor.visit_seq(TupleSeqAccess {
                index: 0,
                pair: (a, b),
            }),
            PklValue::DataSize(d) => visitor.visit_map(DataSizeMapAccess { input: d, state: 0 }),
            PklValue::Map(m) => {
                visitor.visit_map(PklMapAccess::new(&mut Deserializer::from_pkl_map(m)))
            }
        }
    }
}

impl PklValue {
    pub fn into_deserializer(&self) -> PklValueDeserializer {
        PklValueDeserializer(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkl::PklSerialize;
    use rmpv::Value;
    use serde::Deserialize;

    #[cfg(feature = "dhat-heap")]
    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    #[test]
    fn deserialize() {
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

        let eval_module_response = Value::Array(vec![
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

        let pkl_mod = crate::decoder::decode_module(&eval_module_response)
            .expect("failed to evaluate pkl ast");
        let mapped = pkl_mod
            .serialize_pkl_ast()
            .expect("failed to serialize pkl module");

        let deserialized = Config::deserialize(&mut Deserializer::from_pkl_map(&mapped))
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
        assert_eq!(expected, deserialized);
    }

    #[test]
    fn deserialize_time() {
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
            Value::String("file:///Users/testing/code/rust/rpkl/examples/example.pkl".into()),
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
        let expected = Config {
            ip: "127.0.0.1".into(),
            port: 8080,
            birds: vec!["Pigeon".into(), "Hawk".into(), "Penguin".into()],
            database: Database {
                username: "admin".to_owned(),
                password: "secret".to_owned(),
            },
        };
        // let now = std::time::Instant::now();
        let pkl_mod = crate::decoder::decode_module(&ast).expect("failed to evaluate pkl ast");
        let mapped = pkl_mod
            .serialize_pkl_ast()
            .expect("failed to serialize pkl module");

        let deserialized = Config::deserialize(&mut Deserializer::from_pkl_map(&mapped))
            .expect("failed to deserialize");

        // let elapsed = now.elapsed();
        // print_time!(elapsed);
        assert_eq!(expected, deserialized);
    }

    #[test]
    #[cfg(feature = "indexmap")]
    fn test_map_ordering() {
        use crate::internal::IPklValue;
        use crate::internal::ObjectMember;
        use crate::internal::PklPrimitive;

        // Create a sequence of members in a specific order
        // let members = vec![
        //     ObjectMember(
        //         16,
        //         "third".to_string(),
        //         IPklValue::Primitive(PklPrimitive::String("3".to_string())),
        //     ),
        //     ObjectMember(
        //         16,
        //         "first".to_string(),
        //         IPklValue::Primitive(PklPrimitive::String("1".to_string())),
        //     ),
        //     ObjectMember(
        //         16,
        //         "second".to_string(),
        //         IPklValue::Primitive(PklPrimitive::String("2".to_string())),
        //     ),
        // ];

        let members = vec![
            ObjectMember(16, "third".to_string(), PklValue::String("3".to_string())),
            ObjectMember(16, "first".to_string(), PklValue::String("1".to_string())),
            ObjectMember(16, "second".to_string(), PklValue::String("2".to_string())),
        ];

        // Serialize to a map
        let map = members.serialize_pkl_ast().unwrap();

        // Verify the order is preserved
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec!["third", "first", "second"]);

        // Verify values are also in the correct order
        let values: Vec<_> = map.values().filter_map(|v| v.as_str()).collect();
        assert_eq!(values, vec!["3", "1", "2"]);

        // Test that order is preserved through deserialization
        #[derive(Debug, Deserialize)]
        struct OrderTest {
            third: String,
            first: String,
            second: String,
        }

        let mut deserializer = Deserializer::from_pkl_map(&map);
        let deserialized = OrderTest::deserialize(&mut deserializer).unwrap();

        assert_eq!(deserialized.third, "3");
        assert_eq!(deserialized.first, "1");
        assert_eq!(deserialized.second, "2");
    }
}

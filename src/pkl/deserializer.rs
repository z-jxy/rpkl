use std::collections::HashMap;

use crate::pkl::de::{
    DurationDeserializer, DurationMapAccess, RangeDeserializer, RangeMapAccess, TupleDeserializer,
    TupleSeqAccess,
};
use crate::pkl::internal::{self};
use crate::value::datasize::{DataSizeDeserializer, DataSizeMapAccess};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::forward_to_deserialize_any;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

use crate::pkl::{self};
use crate::Value as PklValue;

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
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str option string
        bytes byte_buf unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }

    // fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    // where
    //     V: Visitor<'de>,
    // {
    //     #[cfg(feature = "trace")]
    //     debug!("deserialize_option");

    //     match self.map.get("value") {
    //         Some(value) => visitor.visit_some(value.into_deserializer()),
    //         None => visitor.visit_none(),
    //     }
    // }

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

        // seed.deserialize(&mut *self.de)

        let key = self.keys[self.index - 1];
        if let Some(value) = self.de.map.get(key) {
            #[cfg(feature = "trace")]
            debug!("next_value_seed for key: {:?}: {:?}", key, value);
            seed.deserialize(value.into_deserializer())
            // match value {
            //     PklValue::Int(i) => match i {
            //         internal::Integer::Pos(u) => seed.deserialize((*u).into_deserializer()),
            //         internal::Integer::Neg(n) => seed.deserialize((*n).into_deserializer()),
            //         internal::Integer::Float(f) => seed.deserialize((*f).into_deserializer()),
            //     },
            //     PklValue::String(s) => seed.deserialize(s.as_str().into_deserializer()),

            //     PklValue::List(elements) => {
            //         #[cfg(feature = "trace")]
            //         let _span = span!(Level::INFO, "start parsing list").entered();

            //         #[cfg(feature = "trace")]
            //         debug!("parsing list: {:?}", elements);

            //         let seq = SeqAccessImpl::new(self.de, elements);
            //         let result = seed.deserialize(SeqAccessDeserializer { seq });

            //         #[cfg(feature = "trace")]
            //         _span.exit();

            //         result
            //     }
            //     PklValue::Map(m) => seed.deserialize(&mut Deserializer::from_pkl_map(m)),
            //     PklValue::Boolean(b) => seed.deserialize(b.into_deserializer()),
            //     PklValue::Null => seed.deserialize(().into_deserializer()),
            //     PklValue::Pair(a, b) => {
            //         #[cfg(feature = "trace")]
            //         debug!("pair: {:?}, {:?}", a, b);

            //         seed.deserialize(TupleDeserializer { pair: (&*a, &*b) })
            //     }

            //     PklValue::Duration(duration) => seed.deserialize(DurationDeserializer { duration }),
            //     PklValue::Range(r) => seed.deserialize(RangeDeserializer {
            //         start: &r.start,
            //         end: &r.end,
            //     }),
            //     PklValue::DataSize(d) => seed.deserialize(DataSizeDeserializer { input: &d }),
            //     PklValue::Regex(s) => seed.deserialize(s.as_str().into_deserializer()),
            // }
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
                PklValue::String(s) | PklValue::Regex(s) => {
                    seed.deserialize(s.as_str().into_deserializer()).map(Some)
                }
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
                PklValue::Pair(a, b) => {
                    #[cfg(feature = "trace")]
                    debug!("pair: {:?}, {:?}", a, b);
                    seed.deserialize(TupleDeserializer { pair: (&*a, &*b) })
                        .map(Some)
                }
                PklValue::DataSize(d) => seed
                    .deserialize(DataSizeDeserializer { input: &d })
                    .map(Some),

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
                PklValue::Duration(d) => seed
                    .deserialize(DurationDeserializer { duration: d })
                    .map(Some),
                PklValue::Range(r) => seed
                    .deserialize(RangeDeserializer {
                        start: &r.start,
                        end: &r.end,
                    })
                    .map(Some),
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

impl<'de, 'a> SeqAccess<'de> for PklSeqAccess<'a> {
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

impl<'de, 'a> MapAccess<'de> for PklMapAccess<'a> {
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

        // seed.deserialize(&mut *self.de)

        let key = self.keys[self.index - 1];
        if let Some(value) = self.de.map.get(key) {
            #[cfg(feature = "trace")]
            debug!("next_value_seed for key: {:?}: {:?}", key, value);
            seed.deserialize(value.into_deserializer())
            // match value {
            //     PklValue::Int(i) => match i {
            //         internal::Integer::Pos(u) => seed.deserialize((*u).into_deserializer()),
            //         internal::Integer::Neg(n) => seed.deserialize((*n).into_deserializer()),
            //         internal::Integer::Float(f) => seed.deserialize((*f).into_deserializer()),
            //     },
            //     PklValue::String(s) => seed.deserialize(s.as_str().into_deserializer()),

            //     PklValue::List(elements) => {
            //         #[cfg(feature = "trace")]
            //         let _span = span!(Level::INFO, "start parsing list").entered();

            //         #[cfg(feature = "trace")]
            //         debug!("parsing list: {:?}", elements);

            //         let seq = SeqAccessImpl::new(self.de, elements);
            //         let result = seed.deserialize(SeqAccessDeserializer { seq });

            //         #[cfg(feature = "trace")]
            //         _span.exit();

            //         result
            //     }
            //     PklValue::Map(m) => seed.deserialize(&mut Deserializer::from_pkl_map(m)),
            //     PklValue::Boolean(b) => seed.deserialize(b.into_deserializer()),
            //     PklValue::Null => seed.deserialize(().into_deserializer()),
            //     PklValue::Pair(a, b) => {
            //         #[cfg(feature = "trace")]
            //         debug!("pair: {:?}, {:?}", a, b);

            //         seed.deserialize(TupleDeserializer { pair: (&*a, &*b) })
            //     }

            //     PklValue::Duration(duration) => seed.deserialize(DurationDeserializer { duration }),
            //     PklValue::Range(r) => seed.deserialize(RangeDeserializer {
            //         start: &r.start,
            //         end: &r.end,
            //     }),
            //     PklValue::DataSize(d) => seed.deserialize(DataSizeDeserializer { input: &d }),
            //     PklValue::Regex(s) => seed.deserialize(s.as_str().into_deserializer()),
            // }
        } else {
            Err(Error::Message(format!("no value found for: {key}")))
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

/// Internal deserializer used for deserializing Tuples from PklValue
pub struct PklValueDeserializer<'v>(pub &'v PklValue);

impl<'v, 'de> serde::Deserializer<'de> for PklValueDeserializer<'v> {
    type Error = crate::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
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
                internal::Integer::Pos(u) => visitor.visit_u64(*u),
                internal::Integer::Neg(n) => visitor.visit_i64(*n),
                internal::Integer::Float(f) => visitor.visit_f64(*f),
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
                pair: (&*a, &*b),
            }),
            PklValue::DataSize(d) => visitor.visit_map(DataSizeMapAccess {
                input: &d,
                state: 0,
            }),
            PklValue::Map(m) => {
                // todo!("PklValue::Map deserialization in PklValueDeserializer (lifetime issue)");
                // visitor.visit_map(MapAccessImpl::new(&mut Deserializer::from_pkl_map(m)))
                visitor.visit_map(PklMapAccess::new(&mut Deserializer::from_pkl_map(m)))
            } // PklValue::Duration(_) | PklValue::Pair(_, _) | PklValue::Map(_) => {
              //     unreachable!(
              //         "unhandled deserialization for PklValueDeserializer: {:?}",
              //         self.0
              //     );
              // }
              // PklValue::DataSize(d) => visitor.visit_string(format!("{}{}", d.value(), d.unit())),
        }
    }
}

impl PklValue {
    pub fn into_deserializer(&self) -> PklValueDeserializer {
        PklValueDeserializer(&self)
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
            crate::api::decoder::pkl_eval_module(&ast).expect("failed to evaluate pkl ast");
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

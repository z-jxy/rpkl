use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::{AddAssign, MulAssign, Neg};

use rmpv::Value;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::{
    forward_to_deserialize_any, Deserialize, Deserializer as SerdeDeserializer, Serialize,
};
use tracing::{debug, error, span, trace, Level};
use tracing_subscriber::FmtSubscriber;

use crate::pkl::{self, PklSerialize, PklValue};

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

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub fn from_pkl_map<'a, T>(map: &'a BTreeMap<String, PklValue>) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(&mut Deserializer::from_pkl_map(&map))
}

impl<'de> Deserializer<'de> {
    // Look at the first character in the input without consuming it.
    // fn peek_char(&mut self) -> Result<char> {
    //     self.input.chars().next().ok_or(Error::Eof)
    // }

    // Consume the first character in the input.
    // fn next_char(&mut self) -> Result<char> {
    //     let ch = self.peek_char()?;
    //     self.input = &self.input[ch.len_utf8()..];
    //     Ok(ch)
    // }

    // fn next_field(&mut self) -> {

    // }
    /*
       // Parse the JSON identifier `true` or `false`.
       fn parse_bool(&mut self) -> Result<bool> {
           if self.input.starts_with("true") {
               self.input = &self.input["true".len()..];
               Ok(true)
           } else if self.input.starts_with("false") {
               self.input = &self.input["false".len()..];
               Ok(false)
           } else {
               Err(Error::ExpectedBoolean)
           }
       }

       // Parse a group of decimal digits as an unsigned integer of type T.
       //
       // This implementation is a bit too lenient, for example `001` is not
       // allowed in JSON. Also the various arithmetic operations can overflow and
       // panic or return bogus data. But it is good enough for example code!
       fn parse_unsigned<T>(&mut self) -> Result<T>
       where
           T: AddAssign<T> + MulAssign<T> + From<u8>,
       {
           let mut int = match self.next_char()? {
               ch @ '0'..='9' => T::from(ch as u8 - b'0'),
               _ => {
                   return Err(Error::ExpectedInteger);
               }
           };
           loop {
               match self.input.chars().next() {
                   Some(ch @ '0'..='9') => {
                       self.input = &self.input[1..];
                       int *= T::from(10);
                       int += T::from(ch as u8 - b'0');
                   }
                   _ => {
                       return Ok(int);
                   }
               }
           }
       }

       // Parse a possible minus sign followed by a group of decimal digits as a
       // signed integer of type T.
       fn parse_signed<T>(&mut self) -> Result<T>
       where
           T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
       {
           // Optional minus sign, delegate to `parse_unsigned`, negate if negative.
           unimplemented!()
       }

       // Parse a string until the next '"' character.
       //
       // Makes no attempt to handle escape sequences. What did you expect? This is
       // example code!
       fn parse_string(&mut self) -> Result<&'de str> {
           if self.next_char()? != '"' {
               return Err(Error::ExpectedString);
           }
           match self.input.find('"') {
               Some(len) => {
                   let s = &self.input[..len];
                   self.input = &self.input[len + 1..];
                   Ok(s)
               }
               None => Err(Error::Eof),
           }
       }
    */
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        tuple tuple_struct map struct enum identifier ignored_any
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        trace!("deserialize_any");
        visitor.visit_map(MapAccessImpl::new(self))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        debug!("tracing deserialize_seq");
        // TODO: lifetimes are 'v annoying, refactor to use them properly.
        let values = self.map.values().map(|v| v.to_owned()).collect::<Vec<_>>();
        let seq = SeqAccessImpl::new(self, values);
        visitor.visit_seq(seq)
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
        debug!("tracing next_key_seed");
        // if let Some((key, _)) = self.de.map.iter().next() {
        //     self.key = key;
        //     println!("key: {}", key);
        //     // let key_de = serde_json::Deserializer::from_str(key);
        //     // seed.deserialize(key_de).map(Some)
        //     seed.deserialize(&mut *self.de).map(Some)
        // } else {
        //     error!("no more keys");
        //     Ok(None)
        // }

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
        debug!("tracing next_value_seed");
        // if let Some((_, value)) = self.de.map.iter().next() {
        //     // let value_de = serde_json::Deserializer::from_value(value.clone())?;

        //     seed.deserialize(&mut *self.de)
        // } else {
        //     debug!("no more values");
        //     Err(Error::Eof)
        // }
        // serde_json::Value::Array(vec!["".into()]).deserialize;
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
                    // seed.deserialize(SeqAccessImpl::new(self.de, elements).into_deserializer())
                    let _span = span!(Level::INFO, "start parsing list").entered();
                    // let mut values = vec![];
                    // for v in elements {
                    //     let x = seed.deserialize(v.into_deserializer())?;
                    //     values.push(x);
                    // }
                    let el = elements.iter().map(|v| v.to_owned()).collect::<Vec<_>>();
                    let seq = SeqAccessImpl::new(self.de, el);
                    // let ret = seed.deserialize(seq);
                    // let ret = self.de.deserialize_seq(PklVisitor)?;

                    // let values = elements.iter().map(|v| v).collect::<Vec<_>>();
                    // let seq = SeqAccessImpl::new(self.de, values.as_slice());
                    // let x = seed.deserialize(&mut *seq.de);
                    // seed.deserialize(&mut *seq.de);
                    //let x = seed.deserialize((*values).into_deserializer());
                    // seed.deserialize(deserializer)
                    let zz = seed.deserialize(SeqAccessDeserializer { seq });
                    _span.exit();
                    // x
                    // let seq = SeqAccessImpl::new(self.de, l);

                    // seed.deserialize(&mut *seq.de)

                    zz
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
    elements: Vec<PklValue>,
    index: usize,
}

impl<'a, 'de> SeqAccessImpl<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, elements: Vec<PklValue>) -> Self {
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
                PklValue::List(l) => {
                    let values = l.iter().map(|v| v.to_owned()).collect::<Vec<_>>();
                    let sai = SeqAccessImpl::new(self.de, values);
                    // seed.deserialize(&mut *sai.de).map(Some)
                    // self.de.deserialize_seq(&mut *self.de).map(Some)
                    // seed.deserialize(values.into_deserializer()).map(Some)
                    seed.deserialize(SeqAccessDeserializer { seq: sai })
                        .map(Some)
                }
                PklValue::Map(m) => {
                    todo!();
                    seed.deserialize(&mut Deserializer::from_pkl_map(m))
                        .map(Some)
                }
                PklValue::Boolean(b) => seed.deserialize(b.into_deserializer()).map(Some),
                PklValue::Null => seed.deserialize(().into_deserializer()).map(Some),
            }
        } else {
            Ok(None)
        }
    }
}

/// A deserializer holding a `&PklValue`.
pub struct PklDeserializer<'a, E> {
    value: &'a PklValue,
    marker: PhantomData<E>,
}

impl<'a, E> PklDeserializer<'a, E> {
    #[allow(missing_docs)]
    pub fn new(value: &'a PklValue) -> Self {
        PklDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, E> de::Deserializer<'de> for PklDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            PklValue::Int(i) => match i {
                pkl::Integer::Pos(u) => visitor.visit_u64(*u),
                pkl::Integer::Neg(n) => visitor.visit_i64(*n),
                pkl::Integer::Float(f) => visitor.visit_f64(*f),
            },
            PklValue::String(s) => visitor.visit_str(s),
            PklValue::List(l) => {
                unimplemented!();
                // let values = l.iter().map(|v| v.to_owned()).collect::<Vec<_>>();
                // let seq = SeqAccessImpl::new(self, values);
                // visitor.visit_seq(seq).map_err(|e| E::custom(e))
            }
            PklValue::Map(m) => {
                unimplemented!()
                // let map = MapAccessImpl::new(self);
                // visitor.visit_map(map).map_err(|e| E::custom(e))
            }
            PklValue::Boolean(b) => visitor.visit_bool(*b),
            PklValue::Null => visitor.visit_unit(),
        }
    }

    // fn deserialize_enum<V>(
    //     self,
    //     name: &str,
    //     variants: &'static [&'static str],
    //     visitor: V,
    // ) -> Result<V::Value, Self::Error>
    // where
    //     V: de::Visitor<'de>,
    // {
    //     let _ = name;
    //     let _ = variants;
    //     visitor.visit_enum(self)
    // }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple enum
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de, 'a, E> IntoDeserializer<'de, E> for &'a PklValue
where
    E: de::Error,
{
    type Deserializer = PklDeserializer<'a, E>;

    fn into_deserializer(self) -> PklDeserializer<'a, E> {
        PklDeserializer::new(self)
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { de, first: true }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        // if self.de.peek_char()? == ']' {
        //     return Ok(None);
        // }
        // // Comma is required before every element except the first.
        // if !self.first && self.de.next_char()? != ',' {
        //     return Err(Error::ExpectedArrayComma);
        // }
        todo!();
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        todo!("MapAccess(CommaSeparated): next_key_seed")
        // Check if there are no more entries.
        // if self.de.peek_char()? == '}' {
        //     return Ok(None);
        // }
        // // Comma is required before every entry except the first.
        // if !self.first && self.de.next_char()? != ',' {
        //     return Err(Error::ExpectedMapComma);
        // }
        // self.first = false;
        // // Deserialize a map key.
        // seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        // if self.de.next_char()? != ':' {
        //     return Err(Error::ExpectedMapColon);
        // }
        todo!();
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        let val = seed.deserialize(&mut *self.de)?;
        // Parse the colon separating map key from value.
        // if self.de.next_char()? == ':' {
        //     Ok((val, self))
        // } else {
        //     Err(Error::ExpectedMapColon)
        // }
        todo!()
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

/*

*/

#[test]
fn test_struct() {
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

    // let j = r#"{"int":1,"seq":["a","b"]}"#;

    let x = r#"Array(
        [
            Integer(
                PosInt(
                    1,
                ),
            ),
            String(
                Utf8String {
                    s: Ok(
                        "example",
                    ),
                },
            ),
            String(
                Utf8String {
                    s: Ok(
                        "file:///Users/testing/code/rust/pkl-rs/examples/example.pkl",
                    ),
                },
            ),
            Array(
                [
                    Array(
                        [
                            Integer(
                                PosInt(
                                    16,
                                ),
                            ),
                            String(
                                Utf8String {
                                    s: Ok(
                                        "ip",
                                    ),
                                },
                            ),
                            String(
                                Utf8String {
                                    s: Ok(
                                        "127.0.0.1",
                                    ),
                                },
                            ),
                        ],
                    ),
                    Array(
                        [
                            Integer(
                                PosInt(
                                    16,
                                ),
                            ),
                            String(
                                Utf8String {
                                    s: Ok(
                                        "port",
                                    ),
                                },
                            ),
                            Integer(
                                PosInt(
                                    8080,
                                ),
                            ),
                        ],
                    ),
                    Array(
                        [
                            Integer(
                                PosInt(
                                    16,
                                ),
                            ),
                            String(
                                Utf8String {
                                    s: Ok(
                                        "birds",
                                    ),
                                },
                            ),
                            Array(
                                [
                                    Integer(
                                        PosInt(
                                            5,
                                        ),
                                    ),
                                    Array(
                                        [
                                            String(
                                                Utf8String {
                                                    s: Ok(
                                                        "Pigeon",
                                                    ),
                                                },
                                            ),
                                            String(
                                                Utf8String {
                                                    s: Ok(
                                                        "Hawk",
                                                    ),
                                                },
                                            ),
                                            String(
                                                Utf8String {
                                                    s: Ok(
                                                        "Penguin",
                                                    ),
                                                },
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    Array(
                        [
                            Integer(
                                PosInt(
                                    16,
                                ),
                            ),
                            String(
                                Utf8String {
                                    s: Ok(
                                        "database",
                                    ),
                                },
                            ),
                            Array(
                                [
                                    Integer(
                                        PosInt(
                                            1,
                                        ),
                                    ),
                                    String(
                                        Utf8String {
                                            s: Ok(
                                                "Dynamic",
                                            ),
                                        },
                                    ),
                                    String(
                                        Utf8String {
                                            s: Ok(
                                                "pkl:base",
                                            ),
                                        },
                                    ),
                                    Array(
                                        [
                                            Array(
                                                [
                                                    Integer(
                                                        PosInt(
                                                            16,
                                                        ),
                                                    ),
                                                    String(
                                                        Utf8String {
                                                            s: Ok(
                                                                "username",
                                                            ),
                                                        },
                                                    ),
                                                    String(
                                                        Utf8String {
                                                            s: Ok(
                                                                "admin",
                                                            ),
                                                        },
                                                    ),
                                                ],
                                            ),
                                            Array(
                                                [
                                                    Integer(
                                                        PosInt(
                                                            16,
                                                        ),
                                                    ),
                                                    String(
                                                        Utf8String {
                                                            s: Ok(
                                                                "password",
                                                            ),
                                                        },
                                                    ),
                                                    String(
                                                        Utf8String {
                                                            s: Ok(
                                                                "secret",
                                                            ),
                                                        },
                                                    ),
                                                ],
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )"#;

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

    let pkl_mod = crate::api::pkl_eval_module(ast).expect("failed to evaluate pkl ast");
    let mut mapped = pkl_mod
        .serialize_pkl()
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
    assert_eq!(expected, deserialized);
    // let from_s = from_str(x);
    // println!("{:?}", from_s);
    // assert_eq!(expected, from_s.unwrap());
}
/*
#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    let actual: E = from_str(j).unwrap();
    assert_eq!(expected, actual);

    let j = r#"{"Newtype":1}"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Tuple":[1,2]}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Struct":{"a":1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, from_str(j).unwrap());
}
 */

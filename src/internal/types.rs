use crate::PklSerialize;
use crate::pkl::de::PklVisitor;
use crate::value::{DataSize, PklValue};
use serde::{Deserialize, Serialize};

// use super::visitor::PklVisitor;

/// Represents a member of a `.pkl` object
/// Fields: typeid, identifier, value
#[derive(Debug, Clone, Serialize)]
// pub(crate) struct ObjectMember(pub u64, pub String, pub PklValue);
// TODO: can remove a layer of indirection here,
// but for codegen, we need
pub(crate) struct ObjectMember(pub String, pub PklValue);

impl ObjectMember {
    #[cfg(feature = "codegen")]
    #[inline]
    pub fn get_ident(&self) -> &str {
        self.0.as_str()
    }

    // #[cfg(feature = "codegen")]
    // pub fn get_value(&self) -> &IPklValue {
    //     &self.2
    // }

    // /// Serialize the member to a JSON object
    // ///
    // /// # Returns
    // ///
    // /// A tuple containing the member's identifier and its JSON value
    // pub(crate) fn into_pkl_value(self) -> Result<(String, PklValue)> {
    //     let (_, ident, value) = self.take();
    //     let v = match value {
    //         IPklValue::NonPrimitive(np) => match np {
    //             // serialize nested children
    //             PklNonPrimitive::TypedDynamic(_, _, _, children) => {
    //                 PklValue::Map(children.serialize_pkl_ast()?)
    //             }
    //             PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
    //                 PklValue::List(items.into_iter().collect())
    //             }
    //             PklNonPrimitive::Mapping(_, m) => m,
    //             PklNonPrimitive::Duration(_, d) => PklValue::Duration(d),
    //             PklNonPrimitive::DataSize(_, ds) => PklValue::DataSize(ds),
    //             PklNonPrimitive::Pair(_, a, b) => PklValue::Pair(Box::new(a), Box::new(b)),
    //             PklNonPrimitive::IntSeq(_, a, b) => PklValue::Range(a..b),
    //             PklNonPrimitive::Regex(_, r) => PklValue::Regex(r),
    //         },
    //         IPklValue::Primitive(p) => match p {
    //             PklPrimitive::Int(i) => match i {
    //                 Integer::Pos(u) => PklValue::Int(Integer::Pos(u)),
    //                 Integer::Neg(i) => PklValue::Int(Integer::Neg(i)),
    //                 Integer::Float(f) => PklValue::Int(Integer::Float(f)),
    //             },
    //             PklPrimitive::Float(f) => PklValue::Int(Integer::Float(f)),
    //             PklPrimitive::String(s) => PklValue::String(s),
    //             PklPrimitive::Boolean(b) => PklValue::Boolean(b),
    //             PklPrimitive::Null => PklValue::Null,
    //         },
    //     };

    //     Ok((ident, v))
    // }
}

// #[derive(Debug, Clone, Serialize, PartialEq)]
// struct Pair(pub PklValue, pub PklValue);

#[cfg(test)]
mod test {
    #[test]
    fn deserialize_map() {
        use crate::{Value, internal::Integer};
        let json_data = r#"{"value": 123}"#;
        let value: Value = serde_json::from_str(json_data).expect("Failed to deserialize");
        let map = value.as_map().expect("Expected a map");
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("value").unwrap().as_int().unwrap(),
            &Integer::Pos(123)
        );
    }

    #[test]
    fn deserialize_array() {
        use crate::{Value, internal::Integer};
        let json_data = r#"{"value": [123, 456]}"#;
        let value: Value = serde_json::from_str(json_data).expect("Failed to deserialize");
        let map = value.as_map().expect("Expected a map");
        assert_eq!(map.len(), 1);
        let actual = map.get("value").unwrap().as_array().unwrap();
        assert_eq!(
            *actual,
            vec![Value::Int(Integer::Pos(123)), Value::Int(Integer::Pos(456))]
        );
    }
}

impl From<PklPrimitive> for PklValue {
    fn from(p: PklPrimitive) -> Self {
        match p {
            PklPrimitive::Int(i) => PklValue::Int(i),
            PklPrimitive::Float(f) => PklValue::Int(Integer::Float(f)),
            PklPrimitive::String(s) => PklValue::String(s),
            PklPrimitive::Boolean(b) => PklValue::Boolean(b),
            PklPrimitive::Null => PklValue::Null,
        }
    }
}

impl From<PklNonPrimitive> for PklValue {
    fn from(np: PklNonPrimitive) -> Self {
        match np {
            PklNonPrimitive::TypedDynamic(_, _, _, children) => {
                PklValue::Map(children.serialize_pkl_ast().unwrap())
            }
            PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
                PklValue::List(items.into_iter().collect())
            }
            PklNonPrimitive::Mapping(_, m) => m,
            PklNonPrimitive::Duration(_, d) => PklValue::Duration(d),
            PklNonPrimitive::DataSize(_, ds) => PklValue::DataSize(ds),
            PklNonPrimitive::Pair(_, a, b) => PklValue::Pair(Box::new(a), Box::new(b)),
            PklNonPrimitive::IntSeq(_, a, b) => PklValue::Range(a..b),
            PklNonPrimitive::Regex(_, r) => PklValue::Regex(r),
        }
    }
}

impl<'de> Deserialize<'de> for PklValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<PklValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(PklVisitor)
    }
}

/// Represents an integer in `.pkl`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd, Copy)]
#[serde(untagged)]
// TODO: this will always be a signed integer
pub enum Integer {
    Pos(u64),
    Float(f64),
    Neg(i64),
}

/// Internal struct used for deserializing `.pkl` objects
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum IPklValue {
    Primitive(PklPrimitive),
    NonPrimitive(PklNonPrimitive),
}

impl From<IPklValue> for PklValue {
    fn from(p: IPklValue) -> Self {
        match p {
            IPklValue::Primitive(p) => p.into(),
            IPklValue::NonPrimitive(np) => np.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
/// <https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#primitives>
pub enum PklPrimitive {
    Int(Integer),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
// TODO: do we still need to keep the type id here?
pub(crate) enum PklNonPrimitive {
    /// See `Typed, Dynamic` on <https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#non-primitives>
    ///
    /// ## Fields:
    ///
    /// 0 (slot 1): `type_id` - 0x1
    ///
    /// 1 (slot 2): `type_name` - Fully qualified class name
    ///
    /// 2 (slot 3): `type_version` - Enclosing module URI
    ///
    /// 3 (slot 4): members - array of [Object Members][https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#object-members]
    TypedDynamic(u64, String, String, Vec<ObjectMember>),
    List(u64, Vec<PklValue>),
    Mapping(u64, PklValue),
    Set(u64, Vec<PklValue>),

    Duration(u64, std::time::Duration),
    DataSize(u64, DataSize),
    Pair(u64, PklValue, PklValue),
    /// 0: type id, 1: start, 2: end
    IntSeq(u64, i64, i64),
    Regex(u64, String),
}

/// <https://pkl-lang.org/package-docs/pkl/0.26.1/base/IntSeq>
pub type IntSeq = std::ops::Range<i64>;

impl From<PklNonPrimitive> for IPklValue {
    fn from(np: PklNonPrimitive) -> Self {
        IPklValue::NonPrimitive(np)
    }
}
impl From<PklPrimitive> for IPklValue {
    fn from(p: PklPrimitive) -> Self {
        IPklValue::Primitive(p)
    }
}

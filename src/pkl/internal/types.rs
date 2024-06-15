use std::collections::HashMap;

use crate::PklSerialize;
use crate::Result;
use serde::{Deserialize, Serialize};

use super::visitor::PklVisitor;

/// Represents a member of a `.pkl` object
/// Fields: type_id, identifier, value
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ObjectMember(pub u64, pub String, pub IPklValue);

impl ObjectMember {
    #[cfg(feature = "codegen")]
    pub fn get_ident(&self) -> &str {
        self.1.as_str()
    }

    #[cfg(feature = "codegen")]
    pub fn get_value(&self) -> &IPklValue {
        &self.2
    }

    pub fn take(self) -> (u64, String, IPklValue) {
        (self.0, self.1, self.2)
    }

    /// Serialize the member to a JSON object
    ///
    /// # Returns
    ///
    /// A tuple containing the member's identifier and its JSON value
    pub fn to_pkl_value(self) -> Result<(String, PklValue)> {
        let (_, ident, value) = self.take();
        let v = match value {
            IPklValue::NonPrimitive(np) => match np {
                // serialize nested children
                PklNonPrimitive::TypedDynamic(_, _, _, children) => {
                    PklValue::Map(children.serialize_pkl_ast()?)
                }
                PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
                    PklValue::List(items.into_iter().map(|i| i.into()).collect())
                }
                PklNonPrimitive::Mapping(_, m) => m,
            },
            IPklValue::Primitive(p) => match p {
                PklPrimitive::Int(i) => match i {
                    Integer::Pos(u) => PklValue::Int(Integer::Pos(u)),
                    Integer::Neg(i) => PklValue::Int(Integer::Neg(i)),
                    Integer::Float(f) => PklValue::Int(Integer::Float(f)),
                },
                PklPrimitive::Float(f) => PklValue::Int(Integer::Float(f)),
                PklPrimitive::String(s) => PklValue::String(s.to_string()),
                PklPrimitive::Boolean(b) => PklValue::Boolean(b),
                PklPrimitive::Null => PklValue::Null,
            },
        };

        Ok((ident, v))
    }
}

/// Represents a `.pkl` value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PklValue {
    Map(HashMap<String, PklValue>),
    List(Vec<PklValue>),
    String(String),
    Int(Integer),
    Boolean(bool),
    Null,
}

impl PklValue {
    pub fn as_map(&self) -> Option<&HashMap<String, PklValue>> {
        match self {
            PklValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<PklValue>> {
        match self {
            PklValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<&Integer> {
        match self {
            PklValue::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match self {
            PklValue::Boolean(b) => Some(b),
            _ => None,
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

#[cfg(test)]
mod test {
    #[test]
    fn deserialize_map() {
        use crate::{pkl::internal::Integer, Value};
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
        use crate::{pkl::internal::Integer, Value};
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum Integer {
    Pos(u64),
    Float(f64),
    Neg(i64),
}

impl PklValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PklValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Internal struct used for deserializing `.pkl` objects
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum IPklValue {
    Primitive(PklPrimitive),
    NonPrimitive(PklNonPrimitive),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
/// https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#primitives
pub enum PklPrimitive {
    Int(Integer),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub(crate) enum PklNonPrimitive {
    /// See `Typed, Dynamic` on <https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#non-primitives>
    ///
    /// ## Fields:
    ///
    /// 0 (slot 1): type_id - 0x1
    ///
    /// 1 (slot 2): type_name - Fully qualified class name
    ///
    /// 2 (slot 3): type_version - Enclosing module URI
    ///
    /// 3 (slot 4): members - array of [Object Members][https://pkl-lang.org/main/current/bindings-specification/binary-encoding.html#object-members]
    TypedDynamic(u64, String, String, Vec<ObjectMember>),
    List(u64, Vec<PklPrimitive>),
    Mapping(u64, PklValue),
    Set(u64, Vec<PklPrimitive>),
}

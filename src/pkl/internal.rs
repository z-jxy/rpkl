use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::PklSerialize;

/// Represents a member of a `.pkl` object
/// Fields: type_id, identifier, value
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ObjectMember(pub u64, pub String, pub IPklValue);

impl ObjectMember {
    pub fn get_ident(&self) -> &str {
        self.1.as_str()
    }
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
    pub fn to_pkl_value(self) -> anyhow::Result<(String, PklValue)> {
        let (_, ident, value) = self.take();
        let v = match value {
            IPklValue::NonPrimitive(np) => match np {
                // serialize the children
                PklNonPrimitive::TypedDynamic(_, _, _, children) => {
                    PklValue::Map(children.serialize_pkl_ast()?)
                }
                PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
                    PklValue::List(items.into_iter().map(|i| i.into()).collect())
                }
                PklNonPrimitive::Mapping(_, m) => m,
            },
            // IPklValue::Primitive(p) => serde_json::to_value(p)?,
            IPklValue::Primitive(p) => {
                // p.to_owned(),
                match p {
                    PklPrimitive::Int(i) => match i {
                        Integer::Pos(u) => PklValue::Int(Integer::Pos(u)),
                        Integer::Neg(i) => PklValue::Int(Integer::Neg(i)),
                        Integer::Float(f) => PklValue::Int(Integer::Float(f)),
                    },
                    PklPrimitive::Float(f) => {
                        println!("float: {:?}", f);
                        PklValue::Int(Integer::Float(f))
                    }
                    PklPrimitive::String(s) => PklValue::String(s.to_string()),
                    PklPrimitive::Bool(b) => PklValue::Boolean(b),
                    PklPrimitive::Null => PklValue::Null,
                }
                // PklValue::Map(BTreeMap::new())
            }
        };

        Ok((ident, v))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PklValue {
    Map(BTreeMap<String, PklValue>),
    List(Vec<PklValue>),
    String(String),
    Int(Integer),
    Boolean(bool),
    Null,
    // Container,
}

impl From<PklPrimitive> for PklValue {
    fn from(p: PklPrimitive) -> Self {
        match p {
            PklPrimitive::Int(i) => PklValue::Int(i),
            PklPrimitive::Float(f) => PklValue::Int(Integer::Float(f)),
            PklPrimitive::String(s) => PklValue::String(s),
            PklPrimitive::Bool(b) => PklValue::Boolean(b),
            PklPrimitive::Null => PklValue::Null,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IPklValue {
    Primitive(PklPrimitive),
    NonPrimitive(PklNonPrimitive),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PklPrimitive {
    Int(Integer),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

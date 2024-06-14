use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod non_primitive;
pub mod pkl_mod;
mod serializer;
pub use pkl_mod::PklMod;
pub use serializer::PklSerialize;

/// Represents a member of a `.pkl` object
/// Fields: type_id, identifier, value
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectMember(pub u64, pub String, pub IPklValue);

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
                PklNonPrimitive::TypedDynamic(_, _, _, children) => {
                    // serialize the children
                    PklValue::Map(children.serialize_pkl_ast()?)
                }
                PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
                    let values = items.iter().map(|i| i.to_owned().into()).collect();
                    PklValue::List(values)
                }
                PklNonPrimitive::Mapping(_, m) => {
                    // IPklValue::Primitive(m.to_owned())
                    m
                }
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
pub enum PklNonPrimitive {
    // TypedDynamic(u64, String, String, Vec<ObjectMember>),
    // List(u64, Vec<serde_json::Value>),
    // Mapping(u64, serde_json::Value),
    // Set(u64, Vec<serde_json::Value>),
    TypedDynamic(u64, String, String, Vec<ObjectMember>),
    // TODO: use a serde deserialize
    List(u64, Vec<PklPrimitive>),
    Mapping(u64, PklValue),
    Set(u64, Vec<PklPrimitive>),
}

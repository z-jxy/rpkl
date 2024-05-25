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

    /// Serialize the member to a JSON object
    ///
    /// # Returns
    ///
    /// A tuple containing the member's identifier and its JSON value
    fn to_json(&self) -> anyhow::Result<(String, serde_json::Value)> {
        let v: serde_json::Value = match self.get_value() {
            IPklValue::NonPrimitive(np) => match np {
                PklNonPrimitive::TypedDynamic(_, _, _, children) => {
                    let nested = children.serialize_json()?;
                    nested.into()
                }
                PklNonPrimitive::List(_, items) | PklNonPrimitive::Set(_, items) => {
                    serde_json::Value::Array(items.to_vec())
                }
                PklNonPrimitive::Mapping(_, m) => m.to_owned(),
            },
            IPklValue::Primitive(p) => serde_json::to_value(p)?,
        };
        Ok((self.get_ident().to_owned(), v))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IPklValue {
    Primitive(PklPrimitive),
    NonPrimitive(PklNonPrimitive),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PklPrimitive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PklNonPrimitive {
    TypedDynamic(u64, String, String, Vec<ObjectMember>),
    List(u64, Vec<serde_json::Value>),
    Mapping(u64, serde_json::Value),
    Set(u64, Vec<serde_json::Value>),
}

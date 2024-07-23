use std::collections::HashMap;

use serde::Serialize;

use crate::{pkl::internal::Integer, value::DataSize};

/// Represents a `.pkl` value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PklValue {
    Map(HashMap<String, PklValue>),
    List(Vec<PklValue>),
    /// Represents a regex string
    Regex(String),
    String(String),
    Int(Integer),
    Boolean(bool),
    Duration(std::time::Duration),

    Pair(Box<PklValue>, Box<PklValue>), // requires box to avoid infinite size compiler error
    //
    Range(std::ops::Range<i64>),
    DataSize(DataSize),
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

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PklValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, PklValue::Int(_))
    }

    pub fn is_i64(&self) -> bool {
        matches!(self, PklValue::Int(Integer::Neg(_)))
    }

    pub fn is_u64(&self) -> bool {
        matches!(self, PklValue::Int(Integer::Pos(_)))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, PklValue::Int(Integer::Float(_)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, PklValue::String(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, PklValue::Boolean(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, PklValue::Map(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, PklValue::List(_))
    }
}

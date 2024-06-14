use std::fmt;

use serde::de::{self, Visitor};

use crate::pkl::{Integer, PklValue};

pub struct PklVisitor;

impl<'de> Visitor<'de> for PklVisitor {
    type Value = PklValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        todo!("implement PklVisitor::expecting");
        formatter.write_str("")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_borrowed_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PklValue::String(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v {
            Ok(PklValue::Boolean(true))
        } else {
            Ok(PklValue::Boolean(false))
        }
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PklValue::Int(Integer::Neg(i64::from(value))))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PklValue::Int(Integer::Neg(i64::from(value))))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PklValue::Int(Integer::Neg(value)))
    }
}

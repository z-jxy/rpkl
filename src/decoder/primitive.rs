use crate::{
    pkl::internal::{Integer, PklPrimitive},
    Error, Result,
};

/// decodes a primitive value from `rmpv::Value` to `PklPrimitive`.
pub fn decode_primitive(value: &rmpv::Value) -> Result<PklPrimitive> {
    match value {
        rmpv::Value::String(_) => decode_string(value),
        rmpv::Value::Boolean(b) => Ok(PklPrimitive::Boolean(*b)),
        rmpv::Value::Nil => Ok(PklPrimitive::Null),
        rmpv::Value::Integer(_) => decode_int(value),
        rmpv::Value::F32(f) => Ok(PklPrimitive::Float(f64::from(*f))),
        rmpv::Value::F64(f) => Ok(PklPrimitive::Float(*f)),
        _ => unimplemented!("parse other primitive types. value: {}", value),
    }
}

#[inline]
pub fn decode_string(value: &rmpv::Value) -> Result<PklPrimitive> {
    let Some(s) = value.as_str() else {
        return Err(Error::DecodeError(format!(
            "expected valid UTF-8 string, got {value:?}"
        )));
    };

    Ok(PklPrimitive::String(s.to_string()))
}

#[inline]
pub fn decode_int(value: &rmpv::Value) -> Result<PklPrimitive> {
    if value.is_i64() {
        return Ok(PklPrimitive::Int(Integer::Neg(value.as_i64().unwrap())));
    } else if value.is_u64() {
        return Ok(PklPrimitive::Int(Integer::Pos(value.as_u64().unwrap())));
    } else if value.as_f64().is_some() {
        return Ok(PklPrimitive::Float(value.as_f64().unwrap()));
    }

    Err(Error::DecodeError(format!(
        "expected integer, got {value:?}"
    )))
}

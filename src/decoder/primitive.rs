use crate::{
    Error, Result,
    internal::{Integer, PklPrimitive},
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
        _ => Err(Error::Message(format!(
            "unexpected primitive value, got: {value:?}"
        ))),
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
    if value.is_u64() {
        return Ok(PklPrimitive::Int(Integer::Pos(value.as_u64().unwrap())));
    } else if value.is_i64() {
        return Ok(PklPrimitive::Int(Integer::Neg(value.as_i64().unwrap())));
    } else if value.as_f64().is_some() {
        return Ok(PklPrimitive::Float(value.as_f64().unwrap()));
    }

    Err(Error::DecodeError(format!(
        "expected integer, got {value:?}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::Integer;

    #[test]
    fn decode_int_positive_is_pos_variant() {
        // A small positive integer should use Integer::Pos, not Integer::Neg
        let val = rmpv::Value::Integer(8080.into());
        let result = decode_int(&val).unwrap();
        assert!(
            matches!(result, PklPrimitive::Int(Integer::Pos(8080))),
            "expected Integer::Pos(8080), got {result:?}"
        );
    }

    #[test]
    fn decode_int_zero_is_pos_variant() {
        let val = rmpv::Value::Integer(0.into());
        let result = decode_int(&val).unwrap();
        assert!(matches!(result, PklPrimitive::Int(Integer::Pos(0))));
    }

    #[test]
    fn decode_int_negative_is_neg_variant() {
        let val = rmpv::Value::Integer((-42i64).into());
        let result = decode_int(&val).unwrap();
        assert!(matches!(result, PklPrimitive::Int(Integer::Neg(-42))));
    }

    #[test]
    fn decode_int_large_u64_is_pos_variant() {
        // Values > i64::MAX can only fit in u64
        let large: u64 = u64::MAX;
        let val = rmpv::Value::Integer(rmpv::Integer::from(large));
        let result = decode_int(&val).unwrap();
        assert!(matches!(result, PklPrimitive::Int(Integer::Pos(u64::MAX))));
    }
}

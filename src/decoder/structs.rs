#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
#[cfg(not(feature = "indexmap"))]
use std::collections::HashMap;

use crate::{
    Error, Result, Value as PklValue,
    context::Context,
    decoder::primitive::decode_primitive,
    internal::{IPklValue, ObjectMember, PklNonPrimitive, type_constants},
    utils::macros::_trace,
    value::{DataSize, datasize::DataSizeUnit, value::MapImpl},
};

pub fn decode_object_member(data: &[rmpv::Value]) -> Result<ObjectMember> {
    let (type_id, slots) = decode_object_type_id(data)?;

    match type_id {
        type_constants::OBJECT_MEMBER
        | type_constants::DYNAMIC_MAPPING
        | type_constants::DYNAMIC_LISTING => decode_object_generic(type_id, slots),
        _ => {
            return Err(Error::Message(format!(
                "unexpected type id when decoding object member, got: {type_id}",
            )));
        }
    }
}

/// Decode the first slot of the to get its type ID
#[inline]
fn decode_object_type_id(data: &[rmpv::Value]) -> Result<(u64, &[rmpv::Value])> {
    if data.is_empty() {
        return Err(Error::DecodeError("empty data for object".into()));
    }

    let type_id = data
        .first()
        .and_then(rmpv::Value::as_u64)
        .context("expected type id in object preamble")?;

    Ok((type_id, &data[1..]))
}

/// Decodes the inner member of a pkl object
fn decode_object_generic(_type_id: u64, slots: &[rmpv::Value]) -> Result<ObjectMember> {
    let ident = slots
        .first()
        .and_then(rmpv::Value::as_str)
        .context("expected ident for object")?;

    _trace!("decoding ident {:?}", ident);

    let value = slots
        .get(1)
        .context("[decode_object_generic] expected value")?;

    Ok(ObjectMember(ident.to_owned(), decode_member(value)?.into()))
}

/// helper function to decode a member into an `IPklValue`
#[inline]
fn decode_member(value: &rmpv::Value) -> Result<IPklValue> {
    // if its an array, parse the inner object, otherwise parse the primitive value
    if let Some(array) = value.as_array() {
        Ok(decode_non_primitive(array)?.into())
    } else {
        Ok(decode_primitive(value)?.into())
    }
}

/// decodes non-primitive members of a pkl object
fn decode_non_primitive(slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let (type_id, slots) = decode_object_type_id(slots)?;

    match type_id {
        type_constants::TYPED_DYNAMIC => decode_typed(type_id, slots),
        type_constants::SET => decode_set(type_id, slots),
        type_constants::MAPPING | type_constants::MAP => decode_mapping(type_id, slots),
        type_constants::LIST | type_constants::LISTING => decode_list(type_id, slots),
        type_constants::DURATION => decode_duration(type_id, slots),
        type_constants::DATA_SIZE => decode_datasize(type_id, slots),
        type_constants::PAIR => decode_pair(type_id, slots),
        type_constants::INT_SEQ => decode_intseq(type_id, slots),
        type_constants::REGEX => decode_regex(type_id, slots),
        // decoding is the same as a List<UInt8> type
        type_constants::BYTES => decode_bytes(type_id, slots),

        // afaik, pkl doesn't send this information over in the evaluated data
        type_constants::TYPE_ALIAS => {
            unreachable!("found TYPE_ALIAS in pkl binary data {}", type_id)
        }
        _ => {
            return Err(Error::Message(format!(
                "unexpected type id when decoding non-primitive value, got: {type_id} (needs to be implemented)",
            )));
        }
    }
}

fn decode_typed(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let dyn_ident = slots[0]
        .as_str()
        .context("[typed] expected fully qualified name")?;
    let module_uri = slots[1].as_str().context("[typed] expected module uri")?;
    let Some(members) = slots[2].as_array() else {
        return Err(Error::DecodeError(format!(
            "expected array of object members, got: {:?}",
            slots[2]
        )));
    };

    let decoded_members = members
        .iter()
        .map(|m| {
            let Some(m) = m.as_array() else {
                return Err(Error::DecodeError(format!(
                    "expected array for object member, got {m:?}"
                )));
            };
            decode_object_member(m)
                .map_err(|e| Error::Message(format!("failed to parse pkl object member: {e}")))
        })
        .collect::<Result<Vec<ObjectMember>>>()?;

    Ok(PklNonPrimitive::TypedDynamic(
        type_id,
        dyn_ident.to_owned(),
        module_uri.to_owned(),
        decoded_members,
    ))
}

fn decode_set(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let values = &slots[0];
    let values = values
        .as_array()
        .context("expected array when decoding set")?;

    let mut set_values: Vec<PklValue> = Vec::with_capacity(values.len());

    for v in values {
        set_values.push(decode_member(v)?.into());
    }

    Ok(PklNonPrimitive::Set(type_id, set_values))
}

fn decode_mapping(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let values = &slots[0];

    let Some(values) = values.as_map() else {
        return Err(Error::DecodeError(format!(
            "expected map when decoding mapping, got: {values:?}"
        )));
    };

    #[cfg(feature = "indexmap")]
    let mut mapping: MapImpl<String, PklValue> = IndexMap::with_capacity(values.len());
    #[cfg(not(feature = "indexmap"))]
    let mut mapping: MapImpl<String, PklValue> = HashMap::with_capacity(values.len());

    for (k, v) in values {
        let key = k.as_str().context("expected key for mapping")?;
        let value = decode_member(v)?;
        mapping.insert(key.to_string(), value.into());
    }

    Ok(PklNonPrimitive::Mapping(type_id, PklValue::Map(mapping)))
}

fn decode_list(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    _trace!("slots: {:#?}", slots);

    let values = slots[0].as_array().context(format!(
        "expected array when decoding `List`, got: {slots:?}"
    ))?;

    let mut list_values: Vec<PklValue> = Vec::with_capacity(values.len());

    for v in values {
        list_values.push(decode_member(v)?.into());
    }

    Ok(PklNonPrimitive::List(type_id, list_values))
}

fn decode_bytes(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    _trace!("slots: {:#?}", slots);

    let values = slots[0].as_slice().context(format!(
        "expected binary when decoding `Bytes`, got: {slots:?}"
    ))?;

    Ok(PklNonPrimitive::Bytes(type_id, values.to_vec()))
}

#[inline]
fn decode_pair(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    Ok(PklNonPrimitive::Pair(
        type_id,
        decode_member(&slots[0])?.into(),
        decode_member(&slots[1])?.into(),
    ))
}

#[inline]
fn decode_datasize(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let float = slots[0].as_f64().context("expected float for data size")?;
    let size_unit = slots[1].as_str().context("expected size type")?;
    let ds = DataSize::new(float, DataSizeUnit::from(size_unit));
    Ok(PklNonPrimitive::DataSize(type_id, ds))
}

#[inline]
fn decode_intseq(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let start = slots[0].as_i64().context("expected start for int seq")?;
    let end = slots[1].as_i64().context("expected end for int seq")?;
    let step = slots[2].as_i64().context("expected step for int seq")?;
    Ok(PklNonPrimitive::IntSeq(type_id, start, end, step))
}

fn decode_duration(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let value = slots[0].as_f64().context("expected float for duration")?;
    let unit = slots[1].as_str().context("expected time type")?;

    let nanos_f64 = match unit {
        "ns" => value,
        "us" => value * 1_000.0,
        "ms" => value * 1_000_000.0,
        "s" => value * 1_000_000_000.0,
        "min" => value * 60.0 * 1_000_000_000.0,
        "h" => value * 3_600.0 * 1_000_000_000.0,
        "d" => value * 86_400.0 * 1_000_000_000.0,
        _ => {
            return Err(Error::DecodeError(format!(
                "unknown duration unit: {unit:?}"
            )));
        }
    };

    if !nanos_f64.is_finite() || nanos_f64 < 0.0 {
        // TODO: pkl allows for negative durations
        // supporting this would require adding a new type that can represent negative durations
        return Err(Error::DecodeError(format!(
            "invalid duration value: {value} {unit}"
        )));
    }

    let duration = std::time::Duration::from_nanos(nanos_f64.round() as u64);

    Ok(PklNonPrimitive::Duration(type_id, duration))
}

#[inline]
fn decode_regex(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    slots
        .first()
        .and_then(|v| v.as_str())
        .context("expected pattern for regex")
        .map(|pattern| PklNonPrimitive::Regex(type_id, pattern.to_string()))
}

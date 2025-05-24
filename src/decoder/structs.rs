#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
#[cfg(not(feature = "indexmap"))]
use std::collections::HashMap;

use crate::{
    Error, Result, Value as PklValue,
    context::Context,
    decoder::primitive::decode_primitive,
    internal::{IPklValue, ObjectMember, PklNonPrimitive, type_constants},
    utils,
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
            unimplemented!(
                "implement parse other non-primitive types. type_id: {}\n",
                type_id
            );
        }
    }
}

/// Decode the preamble of an object to get its type ID
#[inline]
fn decode_object_type_id(data: &[rmpv::Value]) -> Result<(u64, &[rmpv::Value])> {
    if data.is_empty() {
        return Err(Error::DecodeError("empty data for object".into()));
    }

    let type_id = data[0]
        .as_u64()
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

        // afaik, pkl doesn't send this information over in the evaluated data
        type_constants::TYPE_ALIAS => {
            unreachable!("found TYPE_ALIAS in pkl binary data {}", type_id)
        }
        _ => {
            unimplemented!("parse other non-primitive types. type_id: {}", type_id);
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

    let values = slots[0]
        .as_array()
        .context(format!("expected array when decoding list, got: {slots:?}"))?;

    let mut list_values: Vec<PklValue> = Vec::with_capacity(values.len());

    for v in values {
        list_values.push(decode_member(v)?.into());
    }

    Ok(PklNonPrimitive::List(type_id, list_values))
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
    // nothing is done with 'step' slot of the int seq structure from pkl
    let start = slots[0].as_i64().context("expected start for int seq")?;
    let end = slots[1].as_i64().context("expected end for int seq")?;
    Ok(PklNonPrimitive::IntSeq(type_id, start, end))
}

fn decode_duration(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    // need u64 to convert to Duration
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let float_time = slots[0].as_f64().context("expected float for duration")? as u64;
    let duration_unit = slots[1].as_str().context("expected time type")?;
    let duration = match duration_unit {
        "min" => {
            let Some(d) = utils::duration::from_mins(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from mins: {float_time}"
                )));
            };
            d
        }
        "h" => {
            let Some(d) = utils::duration::from_hours(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from hours: {float_time}"
                )));
            };
            d
        }
        "d" => {
            let Some(d) = utils::duration::from_days(float_time) else {
                return Err(Error::DecodeError(format!(
                    "failed to parse duration from days: {float_time}"
                )));
            };
            d
        }
        "ns" => std::time::Duration::from_nanos(float_time),
        "us" => std::time::Duration::from_micros(float_time),
        "ms" => std::time::Duration::from_millis(float_time),
        "s" => std::time::Duration::from_secs(float_time),
        _ => {
            return Err(Error::DecodeError(format!(
                "unsupported duration_unit, got {duration_unit:?}"
            )));
        }
    };
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

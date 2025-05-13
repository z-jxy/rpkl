#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
use std::collections::HashMap;

use crate::{
    context::Context,
    decoder::primitive::decode_primitive,
    pkl::internal::{type_constants, IPklValue, ObjectMember, PklNonPrimitive},
    utils,
    utils::macros::_trace,
    value::{datasize::DataSizeUnit, value::MapImpl, DataSize},
    Error, Result, Value as PklValue,
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
fn decode_object_generic(type_id: u64, slots: &[rmpv::Value]) -> Result<ObjectMember> {
    let ident = slots
        .first()
        .and_then(rmpv::Value::as_str)
        .context("expected ident for object")?;

    _trace!("decoding ident {:?}", ident);

    let value = slots
        .get(1)
        .context("[decode_object_generic] expected value")?;

    Ok(ObjectMember(
        type_id,
        ident.to_owned(),
        decode_member(value)?,
    ))
}

/// helper function to decode a member into an `IPklValue`
#[inline]
fn decode_member(value: &rmpv::Value) -> Result<IPklValue> {
    // if its an array, parse the inner object, otherwise parse the primitive value
    if let Some(array) = value.as_array() {
        decode_non_primitive(array)
    } else {
        Ok(decode_primitive(value)?.into())
    }
}

/// evaluates the inner array of a pkl object. used for decoding nested non-primitive types
fn decode_non_primitive(slots: &[rmpv::Value]) -> Result<IPklValue> {
    let type_id = slots[0].as_u64().context("missing type id")?;

    match type_id {
        type_constants::OBJECT_MEMBER => {
            // next slot is the ident,
            // we don't need rn bc it's in the object from the outer scope that called this function
            _trace!(
                "decode_inner_bin_array :: found type const type_constants::OBJECT_MEMBER: {}",
                type_id
            );
            unreachable!("found OBJECT_MEMBER in pkl binary data {}", type_id);
            Ok(IPklValue::Primitive(decode_primitive(&slots[2])?))
        }
        non_prim => {
            _trace!(
                "decode_inner_bin_array :: non prim member found. recurse for type_id: {}",
                crate::pkl::internal::type_constants::pkl_type_id_str(type_id)
            );
            let value = decode_struct(non_prim, &slots[1..])?;
            _trace!("decode_inner_bin_array :: decoded value: {:?}", non_prim);
            Ok(IPklValue::NonPrimitive(value))
        }
    }
}

/// decodes non-primitive members of a pkl object
fn decode_struct(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    match type_id {
        type_constants::TYPED_DYNAMIC => decode_typed(type_id, slots),
        type_constants::SET => decode_set(type_id, slots),
        type_constants::MAPPING | type_constants::MAP => decode_mapping(type_id, slots),
        type_constants::LIST | type_constants::LISTING => decode_list(type_id, slots),
        type_constants::DURATION => decode_duration(type_id, slots),
        type_constants::DATA_SIZE => Ok(decode_datasize(type_id, slots)),
        type_constants::PAIR => decode_pair(type_id, slots),
        type_constants::INT_SEQ => Ok(decode_intseq(type_id, slots)),
        type_constants::REGEX => Ok(decode_regex(type_id, slots)),

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
    let dyn_ident = slots[0].as_str().context("expected fully qualified name")?;
    let module_uri = slots[1].as_str().context("expected module uri")?;
    let members = slots[2].as_array().unwrap_or_else(|| {
        panic!(
            "expected array of abstract member objects, got: {:?}",
            slots[2]
        )
    });

    let members = members
        .iter()
        .map(|m| decode_object_member(m.as_array().unwrap()))
        .collect::<Result<Vec<ObjectMember>>>()?;

    Ok(PklNonPrimitive::TypedDynamic(
        type_id,
        dyn_ident.to_owned(),
        module_uri.to_owned(),
        members,
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
    let mut mapping: MapImpl<String, PklValue> =
        if let Some(size) = values.as_map().map(std::vec::Vec::len) {
            #[cfg(feature = "indexmap")]
            let m = IndexMap::with_capacity(size);
            #[cfg(not(feature = "indexmap"))]
            let m = HashMap::with_capacity(size);
            m
        } else {
            #[cfg(feature = "indexmap")]
            let m = IndexMap::new();
            #[cfg(not(feature = "indexmap"))]
            let m = HashMap::new();
            m
        };
    let values = values.as_map().unwrap();

    for (k, v) in values {
        let key = k.as_str().context("expected key for mapping")?;
        let value = decode_member(v)?;
        mapping.insert(key.to_string(), value.into());
    }

    Ok(PklNonPrimitive::Mapping(type_id, PklValue::Map(mapping)))
}

fn decode_list(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    _trace!("LIST | LISTING: type_id: {}", type_id);
    _trace!("slots: {:#?}", slots);

    let values = &slots[0];
    let values = values
        .as_array()
        .unwrap_or_else(|| panic!("Expected array, got {values:?}"));

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
fn decode_datasize(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    let float = slots[0].as_f64().expect("expected float for data size");
    let size_unit = slots[1].as_str().expect("expected size type");
    let ds = DataSize::new(float, DataSizeUnit::from(size_unit));
    PklNonPrimitive::DataSize(type_id, ds)
}

#[inline]
fn decode_intseq(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    // nothing is done with 'step' slot of the int seq structure from pkl
    let start = slots[0].as_i64().expect("expected start for int seq");
    let end = slots[1].as_i64().expect("expected end for int seq");
    PklNonPrimitive::IntSeq(type_id, start, end)
}

fn decode_duration(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    // need u64 to convert to Duration
    let float_time = slots[0].as_f64().expect("expected float for duration") as u64;
    let duration_unit = slots[1].as_str().expect("expected time type");
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
fn decode_regex(type_id: u64, slots: &[rmpv::Value]) -> PklNonPrimitive {
    let pattern = slots[0].as_str().expect("expected pattern for regex");
    PklNonPrimitive::Regex(type_id, pattern.to_string())
}

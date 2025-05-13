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
    let mut slots = data.iter();

    let type_id = slots
        .next()
        .and_then(|v| v.as_u64())
        .context("expected type id")?;

    match type_id {
        type_constants::OBJECT_MEMBER | type_constants::DYNAMIC_MAPPING => {
            decode_object_generic(type_id, &mut slots)
        }
        type_constants::DYNAMIC_LISTING => decode_dynamic_list(type_id, &mut slots),
        _ => {
            unimplemented!("type_id is not OBJECT_MEMBER, or DYNAMIC_LISTING. implement parse other non-primitive types. type_id: {}\n", type_id);
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

/// Decodes the inner member of a pkl object
fn decode_object_generic(
    type_id: u64,
    slots: &mut std::slice::Iter<rmpv::Value>,
) -> Result<ObjectMember> {
    let ident = slots
        .next()
        .map(|v| {
            v.as_str()
                .unwrap_or_else(|| panic!("expected str for ident, got {v:?}"))
                .to_owned()
        })
        .unwrap();

    #[cfg(feature = "trace")]
    trace!("decoding ident {:?}", ident);

    let value = slots.next().expect("[parse_member_inner] expected value");

    // nested object, map using the outer ident
    if let rmpv::Value::Array(array) = value {
        _trace!("got array, decode inner bin {:?}", ident);
        let pkl_value = decode_bin_array(array)?;
        _trace!(
            "decoding for inner bin `{ident}` is complete: {:?}",
            pkl_value
        );
        return Ok(ObjectMember(type_id, ident, pkl_value));
    }

    let primitive = decode_primitive(value)?;
    Ok(ObjectMember(
        type_id,
        ident,
        IPklValue::Primitive(primitive),
    ))
}

/// this function is used to parse dynmically typed listings
///
/// i.e:
///
/// ```ignore
/// birds = new {
///  "Pigeon"
///  "Hawk"
///  "Penguin"
///  }
/// ```
/// the dynamically typed listings have a different structure than the typed listings
///
fn decode_dynamic_list(
    type_id: u64,
    slots: &mut std::slice::Iter<rmpv::Value>,
) -> Result<ObjectMember> {
    _trace!("parse_dynamic_list_inner: type_id: {}", type_id);
    if type_id != type_constants::DYNAMIC_LISTING {
        todo!(
            "expected DYNAMIC_LISTING ( type_id: {}), got: {}",
            type_constants::DYNAMIC_LISTING,
            type_id
        );
    }

    let index = slots
        .next()
        .and_then(|v| v.as_u64())
        .context("expected index for dynamic list")?;

    let value = slots.next().expect("[parse_member_inner] expected value");

    // nested object, map using the outer ident
    if let rmpv::Value::Array(array) = value {
        let pkl_value = decode_bin_array(array)?;
        return Ok(ObjectMember(type_id, index.to_string(), pkl_value));
    }

    let primitive = decode_primitive(value)?;

    Ok(ObjectMember(
        type_id,
        index.to_string(),
        IPklValue::Primitive(primitive),
    ))
}

/// evaluates the inner binary array of a pkl object. used for decoding nested non-primitive types
fn decode_bin_array(slots: &[rmpv::Value]) -> Result<IPklValue> {
    let type_id = slots[0].as_u64().context("missing type id")?;

    if type_id == type_constants::OBJECT_MEMBER {
        // next slot is the ident,
        // we don't need rn bc it's in the object from the outer scope that called this function
        #[cfg(feature = "trace")]
        trace!(
            "decode_inner_bin_array :: found type const type_constants::OBJECT_MEMBER: {}",
            type_id
        );
        let value = &slots[2];
        let primitive = decode_primitive(value)?;
        return Ok(IPklValue::Primitive(primitive));
    }

    // #[cfg(feature = "trace")]
    _trace!(
        "decode_inner_bin_array :: non prim member found. recurse for type_id: {}",
        pkl_type_id_str(type_id)
    );

    let non_prim = decode_struct(type_id, &slots[1..])?;
    #[cfg(feature = "trace")]
    trace!("decode_inner_bin_array :: decoded value: {:?}", non_prim);

    Ok(IPklValue::NonPrimitive(non_prim))
}

fn decode_typed(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    let dyn_ident = slots[0].as_str().expect("expected fully qualified name");
    let module_uri = slots[1].as_str().expect("expected module uri");
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
    let values = values.as_array().unwrap().to_vec();

    let mut set_values = vec![];

    for v in values.iter() {
        if let Some(array) = v.as_array() {
            _trace!("inserting values into set");
            let decoded_value = decode_bin_array(array)?;
            let pkl_value: PklValue = decoded_value.into();

            set_values.push(pkl_value);
        } else {
            let prim = decode_primitive(v)?;
            set_values.push(prim.into());
        }
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
    for (k, v) in values.iter() {
        let key = k.as_str().expect("expected key for mapping");
        if let Some(array) = v.as_array() {
            // add the inner object
            _trace!("inserting fields into mapping");
            mapping.insert(key.to_string(), decode_bin_array(array)?.into());
        } else {
            mapping.insert(key.to_string(), decode_primitive(v)?.into());
        }
    }
    Ok(PklNonPrimitive::Mapping(type_id, PklValue::Map(mapping)))
}

fn decode_list(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    _trace!("LIST | LISTING: type_id: {}", type_id);
    _trace!("slots: {:#?}", slots);

    let values = &slots[0];
    let values = values
        .as_array()
        .unwrap_or_else(|| panic!("Expected array, got {values:?}"))
        .to_vec();

    let mut list_values = Vec::with_capacity(values.len());

    for v in values.iter() {
        let value = match v {
            // decode the inner object
            rmpv::Value::Array(array) => decode_bin_array(array)?.into(),
            _ => decode_primitive(v)?.into(),
        };
        list_values.push(value);
    }

    Ok(PklNonPrimitive::List(type_id, list_values))
}

fn decode_pair(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    // if its an array, parse the inner object, otherwise parse the primitive value
    let first_val: PklValue = if let Some(array) = slots[0].as_array() {
        decode_bin_array(array)?.into()
    } else {
        decode_primitive(&slots[0])?.into()
    };

    let second_val: PklValue = if let Some(array) = slots[1].as_array() {
        decode_bin_array(array)?.into()
    } else {
        decode_primitive(&slots[1])?.into()
    };

    Ok(PklNonPrimitive::Pair(type_id, first_val, second_val))
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

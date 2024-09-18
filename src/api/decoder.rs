use std::collections::HashMap;

use crate::context::Context;
use crate::error::{Error, Result};

#[allow(unused_imports)]
use crate::pkl::internal::type_constants::{self, pkl_type_id_str};

use crate::pkl::{
    self,
    internal::{IPklValue, ObjectMember, PklNonPrimitive, PklPrimitive},
    PklMod,
};

use crate::utils;
use crate::utils::macros::_trace;
use crate::value::{
    datasize::{DataSize, DataSizeUnit},
    PklValue,
};

#[cfg(feature = "trace")]
use tracing::trace;

/// decodes the inner member of a pkl object
fn decode_member_inner(
    type_id: u64,
    slots: &mut std::slice::Iter<rmpv::Value>,
) -> Result<ObjectMember> {
    let ident = slots
        .next()
        .map(|v| {
            v.as_str()
                .expect(&format!("expected str for ident, got {:?}", v))
                .to_owned()
        })
        .unwrap();

    #[cfg(feature = "trace")]
    trace!("decoding ident {:?}", ident);

    let value = slots.next().expect("[parse_member_inner] expected value");

    // nested object, map using the outer ident
    if let rmpv::Value::Array(array) = value {
        _trace!("got array, decode inner bin {:?}", ident);
        let pkl_value = decode_inner_bin_array(&array)?;
        _trace!(
            "decoding for inner bin `{ident}` is complete: {:?}",
            pkl_value
        );
        return Ok(ObjectMember(type_id, ident, pkl_value));
    }

    let primitive = decode_primitive_member(value)?;
    return Ok(ObjectMember(
        type_id,
        ident,
        IPklValue::Primitive(primitive),
    ));
}

/// decodes non-primitive members of a pkl object
fn decode_non_prim_member(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
    match type_id {
        type_constants::TYPED_DYNAMIC => {
            let dyn_ident = slots[0].as_str().expect("expected fully qualified name");

            let module_uri = slots[1].as_str().expect("expected module uri");

            let members = slots[2].as_array().expect(&format!(
                "expected array of abstract member objects, got: {:?}",
                slots[2]
            ));

            let members = members
                .iter()
                .map(|m| decode_pkl_obj_member(&m.as_array().unwrap()))
                .collect::<Result<Vec<ObjectMember>>>()?;

            return Ok(PklNonPrimitive::TypedDynamic(
                type_id,
                dyn_ident.to_owned(),
                module_uri.to_owned(),
                members,
            ));
        }
        type_constants::SET => {
            let values = &slots[0];
            let values = values.as_array().unwrap().to_vec();

            let mut set_values = vec![];

            for v in values.iter() {
                if let Some(array) = v.as_array() {
                    _trace!("inserting values into set");
                    let decoded_value = decode_inner_bin_array(array)?;
                    let pkl_value: PklValue = decoded_value.into();

                    set_values.push(pkl_value);
                } else {
                    let prim = decode_primitive_member(v)?;
                    set_values.push(prim.into());
                }
            }

            return Ok(PklNonPrimitive::Set(type_id, set_values));
        }
        type_constants::MAPPING | type_constants::MAP => {
            let values = &slots[0];
            let mut mapping: HashMap<String, PklValue> = HashMap::new();
            let values = values.as_map().unwrap();
            for (k, v) in values.iter() {
                let key = k.as_str().expect("expected key for mapping");
                if let Some(array) = v.as_array() {
                    // add the inner object
                    _trace!("inserting fields into mapping");
                    let decoded_value = decode_inner_bin_array(array)?;
                    let pkl_value: PklValue = decoded_value.into();
                    mapping.insert(key.to_string(), pkl_value);
                } else {
                    mapping.insert(key.to_string(), decode_primitive_member(v)?.into());
                }
            }
            return Ok(PklNonPrimitive::Mapping(type_id, PklValue::Map(mapping)));
        }

        type_constants::LIST | type_constants::LISTING => {
            #[cfg(feature = "trace")]
            {
                trace!("LIST | LISTING: type_id: {}", type_id);
                trace!("slots: {:#?}", slots);
            }

            let values = &slots[0];
            let values = values
                .as_array()
                .expect(&format!("Expected array, got {:?}", values))
                .to_vec();

            let mut list_values = vec![];

            for v in values.iter() {
                let value = decode_prim_or_non_prim(v)?;
                list_values.push(value);
            }

            return Ok(PklNonPrimitive::List(type_id, list_values));
        }

        type_constants::DURATION => {
            // need u64 to convert to Duration
            let float_time = slots[0].as_f64().expect("expected float for duration") as u64;
            let duration_unit = slots[1].as_str().expect("expected time type");
            let duration = match duration_unit {
                "min" => {
                    let Some(d) = utils::duration::from_mins(float_time) else {
                        return Err(Error::ParseError(format!(
                            "failed to parse duration from mins: {}",
                            float_time
                        )));
                    };
                    d
                }
                "h" => {
                    let Some(d) = utils::duration::from_hours(float_time) else {
                        return Err(Error::ParseError(format!(
                            "failed to parse duration from hours: {}",
                            float_time
                        )));
                    };
                    d
                }
                "d" => {
                    let Some(d) = utils::duration::from_days(float_time) else {
                        return Err(Error::ParseError(format!(
                            "failed to parse duration from days: {}",
                            float_time
                        )));
                    };
                    d
                }
                "ns" => std::time::Duration::from_nanos(float_time),
                "us" => std::time::Duration::from_micros(float_time),
                "ms" => std::time::Duration::from_millis(float_time),
                "s" => std::time::Duration::from_secs(float_time),
                _ => {
                    return Err(Error::ParseError(format!(
                        "unsupported duration_unit, got {:?}",
                        duration_unit
                    )));
                }
            };
            return Ok(PklNonPrimitive::Duration(type_id, duration));
        }

        type_constants::DATA_SIZE => {
            let float = slots[0].as_f64().expect("expected float for data size");
            let size_unit = slots[1].as_str().expect("expected size type");

            let ds = DataSize::new(float, DataSizeUnit::from(size_unit));

            return Ok(PklNonPrimitive::DataSize(type_id, ds));
        }
        type_constants::PAIR => {
            // if its an array, parse the inner object, otherwise parse the primitive value
            let first_val: PklValue = if let Some(array) = slots[0].as_array() {
                decode_inner_bin_array(array)?.into()
            } else {
                decode_primitive_member(&slots[0])?.into()
            };

            let second_val: PklValue = if let Some(array) = slots[1].as_array() {
                decode_inner_bin_array(array)?.into()
            } else {
                decode_primitive_member(&slots[1])?.into()
            };

            return Ok(PklNonPrimitive::Pair(type_id, first_val, second_val));
        }
        type_constants::INT_SEQ => {
            // nothing is done with 'step' slot of the int seq structure from pkl
            let start = slots[0].as_i64().expect("expected start for int seq");
            let end = slots[1].as_i64().expect("expected end for int seq");
            return Ok(PklNonPrimitive::IntSeq(type_id, start, end));
        }

        type_constants::REGEX => {
            let pattern = slots[0].as_str().expect("expected pattern for regex");
            return Ok(PklNonPrimitive::Regex(type_id, pattern.to_string()));
        }

        type_constants::TYPE_ALIAS => {
            unreachable!("found TYPE_ALIAS in pkl binary data {}", type_id);
        }
        _ => {
            todo!("parse other non-primitive types. type_id: {}", type_id);
        }
    }
}

/// decodes the inner binary array of a pkl object into a PklValue
fn decode_prim_or_non_prim(value: &rmpv::Value) -> Result<PklValue> {
    match value {
        rmpv::Value::Array(array) => {
            let inner = decode_inner_bin_array(array)?;
            Ok(inner.into())
        }
        _ => {
            let prim = decode_primitive_member(value)?;
            Ok(prim.into())
        }
    }
}

/// decodes primitive members of a pkl object
fn decode_primitive_member(value: &rmpv::Value) -> Result<PklPrimitive> {
    match value {
        rmpv::Value::String(s) => {
            let Some(s) = s.as_str() else {
                return Err(Error::ParseError(format!(
                    "expected valid UTF-8 string, got {:?}",
                    s
                )));
            };
            return Ok(PklPrimitive::String(s.to_string()));
        }
        rmpv::Value::Boolean(b) => Ok(PklPrimitive::Boolean(b.to_owned())),
        rmpv::Value::Nil => Ok(PklPrimitive::Null),
        rmpv::Value::Integer(n) => {
            if n.is_i64() {
                Ok(PklPrimitive::Int(pkl::internal::Integer::Neg(
                    n.as_i64().unwrap(),
                )))
            } else if n.is_u64() {
                Ok(PklPrimitive::Int(pkl::internal::Integer::Pos(
                    n.as_u64().unwrap(),
                )))
            } else if n.as_f64().is_some() {
                Ok(PklPrimitive::Float(n.as_f64().unwrap()))
            } else {
                return Err(Error::ParseError(format!("expected integer, got {:?}", n)));
            }
        }
        rmpv::Value::F32(f) => Ok(PklPrimitive::Float(*f as f64)),
        rmpv::Value::F64(f) => Ok(PklPrimitive::Float(*f)),

        _ => {
            todo!("parse other primitive types. value: {}", value);
        }
    }
}

/// evaluates the inner binary array of a pkl object. used for decoding nested non-primitive types
fn decode_inner_bin_array(slots: &[rmpv::Value]) -> Result<IPklValue> {
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
        let primitive = decode_primitive_member(value)?;
        return Ok(IPklValue::Primitive(primitive));
    }

    // #[cfg(feature = "trace")]
    _trace!(
        "decode_inner_bin_array :: non prim member found. recurse for type_id: {}",
        pkl_type_id_str(type_id)
    );

    let non_prim = decode_non_prim_member(type_id, &slots[1..])?;
    #[cfg(feature = "trace")]
    trace!("decode_inner_bin_array :: decoded value: {:?}", non_prim);

    Ok(IPklValue::NonPrimitive(non_prim))
}

fn decode_pkl_obj_member(data: &[rmpv::Value]) -> Result<ObjectMember> {
    let mut slots = data.iter();

    let type_id = slots
        .next()
        .and_then(|v| v.as_u64())
        .context("expected type id")?;

    match type_id {
        type_constants::OBJECT_MEMBER | type_constants::DYNAMIC_MAPPING => {
            return decode_member_inner(type_id, &mut slots);
        }
        type_constants::DYNAMIC_LISTING => {
            return decode_dynamic_list_inner(type_id, &mut slots);
        }
        _ => {
            todo!("type_id is not OBJECT_MEMBER, or DYNAMIC_LISTING. implement parse other non-primitive types. type_id: {}\n", type_id);
        }
    }
}

/// this function is used to parse dynmically typed listings

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
fn decode_dynamic_list_inner(
    type_id: u64,
    slots: &mut std::slice::Iter<rmpv::Value>,
) -> Result<ObjectMember> {
    /// TODO: there has to be a bug here

    #[cfg(feature = "trace")]
    trace!("parse_dynamic_list_inner: type_id: {}", type_id);
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
        let pkl_value = decode_inner_bin_array(&array)?;
        return Ok(ObjectMember(type_id, index.to_string(), pkl_value));
    }

    let primitive = decode_primitive_member(value)?;

    Ok(ObjectMember(
        type_id,
        index.to_string(),
        IPklValue::Primitive(primitive),
    ))
}

/// For internal use
pub fn pkl_eval_module(decoded: &rmpv::Value) -> Result<PklMod> {
    let root = decoded.as_array().unwrap();
    let module_name = root.get(1).context("expected root level module name")?;
    let module_uri = root.get(2).context("expected root level module uri")?;
    let children = root.last().context("expected children")?;

    let pkl_module = children.as_array().context("expected array of children")?;

    // parse the members of the module
    let members = pkl_module
        .iter()
        .map(|f| {
            decode_pkl_obj_member(f.as_array().unwrap())
                .map_err(|e| Error::Message(format!("failed to parse pkl object member: {}", e)))
        })
        .collect::<Result<Vec<ObjectMember>>>()?;

    Ok(PklMod {
        _module_name: module_name
            .as_str()
            .context("PklMod name is not valid utf8")?
            .to_string(),
        _module_uri: module_uri
            .as_str()
            .context("PklMod uri is not valid utf8")?
            .to_string(),
        members,
    })
}

use std::collections::HashMap;

use crate::context::Context;
use crate::error::{Error, Result};
use crate::pkl::internal::type_constants;
use crate::pkl::{
    self,
    internal::{IPklValue, ObjectMember, PklNonPrimitive, PklPrimitive, PklValue},
    PklMod,
};

#[cfg(feature = "trace")]
use tracing::trace;

/// parses the inner member of a pkl object
fn parse_member_inner(
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

    let value = slots.next().expect("[parse_member_inner] expected value");

    // nested object, map using the outer ident
    if let rmpv::Value::Array(array) = value {
        let pkl_value = eval_inner_bin_array(&array)?;
        return Ok(ObjectMember(type_id, ident, pkl_value));
    }

    let primitive = parse_primitive_member(value)?;

    Ok(ObjectMember(
        type_id,
        ident,
        IPklValue::Primitive(primitive),
    ))
}

/// parses non-primitive members of a pkl object
fn parse_non_prim_member(type_id: u64, slots: &[rmpv::Value]) -> Result<PklNonPrimitive> {
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
                .map(|m| parse_pkl_obj_member(&m.as_array().unwrap()))
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
            let values = values
                .iter()
                .map(|v| parse_primitive_member(v))
                .collect::<Result<Vec<PklPrimitive>>>()?;
            return Ok(PklNonPrimitive::Set(type_id, values));
        }
        type_constants::MAPPING | type_constants::MAP => {
            let values = &slots[0];
            let mut mapping: HashMap<String, PklValue> = HashMap::new();
            let values = values.as_map().unwrap();
            for (k, v) in values.iter() {
                let key = k.as_str().expect("expected key for mapping");
                if let Some(array) = v.as_array() {
                    // parse the inner object
                    if let IPklValue::NonPrimitive(PklNonPrimitive::TypedDynamic(
                        _,
                        _,
                        _,
                        members,
                    )) = eval_inner_bin_array(array)?
                    {
                        let mut fields = HashMap::new();
                        for member in members {
                            let (ident, value) = member.to_pkl_value()?;
                            fields.insert(ident, value);
                        }

                        mapping.insert(key.to_string(), PklValue::Map(fields));
                    }
                } else {
                    mapping.insert(key.to_string(), parse_primitive_member(v)?.into());
                }
            }
            return Ok(PklNonPrimitive::Mapping(type_id, PklValue::Map(mapping)));
        }

        type_constants::LIST | type_constants::LISTING => {
            let values = &slots[0];
            let values = values
                .as_array()
                .expect(&format!("Expected array, got {:?}", values))
                .to_vec();
            let values = values
                .iter()
                .map(|v| parse_primitive_member(v))
                .collect::<Result<Vec<PklPrimitive>>>()?;

            return Ok(PklNonPrimitive::List(type_id, values));
        }

        type_constants::DURATION => {
            println!("slots: {:?}", slots);
            // need u64 to convert to Duration
            let float_time = slots[0].as_f64().expect("expected float for duration") as u64;
            let duration_unit = slots[1].as_str().expect("expected time type");
            let duration = match duration_unit {
                "min" => std::time::Duration::from_mins(float_time),
                "h" => std::time::Duration::from_hours(float_time),
                "d" => std::time::Duration::from_days(float_time),
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
            println!("duration: {:?}", duration);
            return Ok(PklNonPrimitive::Duration(type_id, duration));
            // todo!("parse duration type");
        }

        type_constants::DATA_SIZE
        | type_constants::PAIR
        | type_constants::INT_SEQ
        | type_constants::REGEX
        | type_constants::TYPE_ALIAS => {
            todo!("type {} cannot be rendered as json", type_id);
        }
        _ => {
            todo!("parse other non-primitive types. type_id: {}", type_id);
        }
    }
}

/// parses primitive members of a pkl object
fn parse_primitive_member(value: &rmpv::Value) -> Result<PklPrimitive> {
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
        _ => {
            todo!("parse other primitive types. value: {}", value);
        }
    }
}

/// evaluates the inner binary array of a pkl object
fn eval_inner_bin_array(slots: &[rmpv::Value]) -> Result<IPklValue> {
    let type_id = slots[0].as_u64().context("missing type id")?;

    if type_id == type_constants::OBJECT_MEMBER {
        // next slot is the ident,
        // we don't need rn bc it's in the object from the outer scope that called this function
        let value = &slots[2];
        let primitive = parse_primitive_member(value)?;
        return Ok(IPklValue::Primitive(primitive));
    }

    let non_prim = parse_non_prim_member(type_id, &slots[1..])?;
    Ok(IPklValue::NonPrimitive(non_prim))
}

fn parse_pkl_obj_member(data: &[rmpv::Value]) -> Result<ObjectMember> {
    let mut slots = data.iter();

    let type_id = slots
        .next()
        .and_then(|v| v.as_u64())
        .context("expected type id")?;

    match type_id {
        type_constants::OBJECT_MEMBER | type_constants::DYNAMIC_MAPPING => {
            return parse_member_inner(type_id, &mut slots);
        }
        type_constants::DYNAMIC_LISTING => {
            return parse_dynamic_list_inner(type_id, &mut slots);
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
fn parse_dynamic_list_inner(
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
        let pkl_value = eval_inner_bin_array(&array)?;
        return Ok(ObjectMember(type_id, index.to_string(), pkl_value));
    }

    let primitive = parse_primitive_member(value)?;

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

    let members = pkl_module
        .iter()
        .map(|f| {
            parse_pkl_obj_member(f.as_array().unwrap())
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

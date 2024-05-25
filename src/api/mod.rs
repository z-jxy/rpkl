use serde_json::Value;

use crate::pkl::{IPklValue, ObjectMember, PklMod, PklNonPrimitive, PklPrimitive};
pub mod evaluator;
pub mod loader;
pub use evaluator::Evaluator;

use crate::pkl::non_primitive::{self};

fn parse_member_inner(
    type_id: u64,
    slots: &mut std::slice::Iter<Value>,
) -> anyhow::Result<ObjectMember> {
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
    if let Value::Array(array) = value {
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

fn parse_non_prim_member(type_id: u64, slots: &[Value]) -> anyhow::Result<PklNonPrimitive> {
    match type_id {
        non_primitive::code::TYPED_DYNAMIC => {
            let dyn_ident = slots[0].as_str().expect("expected fully qualified name");

            let module_uri = slots[1].as_str().expect("expected module uri");

            let members = slots[2].as_array().expect(&format!(
                "expected array of abstract member objects, got: {:?}",
                slots[2]
            ));

            let members = members
                .iter()
                .map(|m| parse_pkl_obj_member(&m.as_array().unwrap()))
                .collect::<anyhow::Result<Vec<ObjectMember>>>()?;

            return Ok(PklNonPrimitive::TypedDynamic(
                type_id,
                dyn_ident.to_owned(),
                module_uri.to_owned(),
                members,
            ));
        }
        non_primitive::code::SET => {
            let values = &slots[0];
            let values = values.as_array().unwrap().to_vec();
            return Ok(PklNonPrimitive::Set(type_id, values));
        }
        non_primitive::code::MAPPING | non_primitive::code::MAP => {
            let values = &slots[0];
            let mut mapping = serde_json::Map::new();
            let values = values.as_object().unwrap();
            for (k, v) in values.iter() {
                if let Some(array) = v.as_array() {
                    // parse the inner object
                    if let IPklValue::NonPrimitive(PklNonPrimitive::TypedDynamic(
                        _,
                        _,
                        _,
                        members,
                    )) = &eval_inner_bin_array(array)?
                    {
                        let mut fields = serde_json::Map::new();
                        for member in members {
                            fields.insert(
                                member.get_ident().to_string(),
                                serde_json::to_value(member.get_value())?,
                            );
                        }
                        mapping.insert(k.to_owned(), serde_json::to_value(fields)?);
                    }
                } else {
                    mapping.insert(k.to_owned(), v.to_owned());
                }
            }

            return Ok(PklNonPrimitive::Mapping(
                type_id,
                serde_json::Value::Object(mapping),
            ));
        }

        non_primitive::code::LIST | non_primitive::code::LISTING => {
            let values = &slots[0];
            // println!("values: {:?}", values);
            let values = values
                .as_array()
                .expect(&format!("Expected array, got {:?}", values))
                .to_vec();

            return Ok(PklNonPrimitive::List(type_id, values));
        }
        non_primitive::code::DURATION
        | non_primitive::code::DATA_SIZE
        | non_primitive::code::PAIR
        | non_primitive::code::INT_SEQ
        | non_primitive::code::REGEX
        | non_primitive::code::TYPE_ALIAS => {
            todo!("type {} cannot be rendered as json", type_id);
        }
        _ => {
            todo!("parse other non-primitive types. type_id: {}", type_id);
        }
    }
}

fn parse_primitive_member(value: &Value) -> anyhow::Result<PklPrimitive> {
    match value {
        Value::String(s) => Ok(PklPrimitive::String(s.to_owned())),
        Value::Bool(b) => Ok(PklPrimitive::Bool(b.to_owned())),
        Value::Null => Ok(PklPrimitive::Null),
        Value::Number(n) => {
            if n.is_f64() {
                Ok(PklPrimitive::Float(n.as_f64().unwrap()))
            } else {
                Ok(PklPrimitive::Int(n.as_i64().unwrap()))
            }
        }
        _ => {
            todo!("parse other primitive types. value: {}", value);
        }
    }
}

fn eval_inner_bin_array(slots: &[Value]) -> anyhow::Result<IPklValue> {
    let type_id = slots
        // .next()
        [0]
    .as_u64()
    // .map(|v| v.as_u64().expect(&format!("expected type id, got {:?}", v)))
    .expect("missing type id");

    if type_id == non_primitive::code::OBJECT_MEMBER {
        // next slot is the ident, we don't need rn bc it's in the outer object
        let value = &slots[2];
        let primitive = parse_primitive_member(value)?;
        return Ok(IPklValue::Primitive(primitive));
    }

    let non_prim = parse_non_prim_member(type_id, &slots[1..])?;
    Ok(IPklValue::NonPrimitive(non_prim))
}

fn parse_pkl_obj_member(data: &[Value]) -> anyhow::Result<ObjectMember> {
    let mut slots = data.iter();
    let type_id = slots
        .next()
        .map(|v| v.as_u64().expect(&format!("expected type id, got {:?}", v)))
        .expect("missing type id");

    if type_id != non_primitive::code::OBJECT_MEMBER {
        todo!(
            "expected OBJECT_MEMBER ( type_id: {}), got: {}",
            non_primitive::code::OBJECT_MEMBER,
            type_id
        );
    }

    return parse_member_inner(type_id, &mut slots);
}

pub fn pkl_eval_module(decoded: Value) -> anyhow::Result<PklMod> {
    let root = decoded.as_array().unwrap();
    let module_name = root.get(1).expect("expected root level module name");
    let module_uri = root.get(2).expect("expected root level module uri");
    let children = root.last().expect("expected children");

    let pkl_module: Vec<Value> = serde_json::from_value(children.to_owned())?;

    let members = pkl_module
        .iter()
        .map(|f| parse_pkl_obj_member(f.as_array().unwrap()))
        .collect::<anyhow::Result<Vec<ObjectMember>>>()?;

    Ok(PklMod {
        _module_name: module_name.as_str().unwrap().to_string(),
        _module_uri: module_uri.as_str().unwrap().to_string(),
        members,
    })
}

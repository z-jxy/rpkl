use std::collections::HashMap;

use crate::Result;

use super::{
    internal::{ObjectMember, PklValue},
    PklMod,
};

pub trait PklSerialize {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>>;
}

impl PklSerialize for Vec<ObjectMember> {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>> {
        serialize_members(self)
    }
}

impl PklSerialize for PklMod {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>> {
        serialize_members(self.members)
    }
}

#[inline]
// serialize the members of a into a hashmap
fn serialize_members<T: IntoIterator<Item = ObjectMember>>(
    members: T,
) -> Result<HashMap<String, PklValue>> {
    let mut pkl_object = HashMap::new();

    for member in members {
        let (k, v) = member.to_pkl_value()?;
        pkl_object.insert(k, v);
    }

    Ok(pkl_object)
}

use std::collections::HashMap;

use crate::Result;

use super::{internal::ObjectMember, PklMod};
use crate::Value as PklValue;

pub trait PklSerialize {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>>;
}

impl PklSerialize for Vec<ObjectMember> {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>> {
        let size_hint = self.len();
        serialize_members(self, Some(size_hint))
    }
}

impl PklSerialize for PklMod {
    fn serialize_pkl_ast(self) -> Result<HashMap<String, PklValue>> {
        let size_hint = self.members.len();
        serialize_members(self.members, Some(size_hint))
    }
}

#[inline]
// serialize the members of a into a hashmap
fn serialize_members<T: IntoIterator<Item = ObjectMember>>(
    members: T,
    size_hint: Option<usize>,
) -> Result<HashMap<String, PklValue>> {
    let mut pkl_object = if let Some(size_hint) = size_hint {
        HashMap::with_capacity(size_hint)
    } else {
        HashMap::new()
    };

    for member in members {
        let (k, v) = member.to_pkl_value()?;
        pkl_object.insert(k, v);
    }

    Ok(pkl_object)
}

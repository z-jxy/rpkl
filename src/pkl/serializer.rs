#[cfg(feature = "indexmap")]
use indexmap::IndexMap;
#[cfg(not(feature = "indexmap"))]
use std::collections::HashMap;

use crate::Result;
use crate::internal::ObjectMember;
use crate::value::value::MapImpl;

use super::PklMod;
use crate::Value as PklValue;

pub trait PklSerialize {
    fn serialize_pkl_ast(self) -> Result<MapImpl<String, PklValue>>;
}

impl PklSerialize for Vec<ObjectMember> {
    fn serialize_pkl_ast(self) -> Result<MapImpl<String, PklValue>> {
        let size_hint = self.len();
        serialize_members(self, Some(size_hint))
    }
}

impl PklSerialize for PklMod {
    fn serialize_pkl_ast(self) -> Result<MapImpl<String, PklValue>> {
        let size_hint = self.members.len();
        serialize_members(self.members, Some(size_hint))
    }
}

#[inline]
// serialize the members of a into a hashmap
fn serialize_members<T: IntoIterator<Item = ObjectMember>>(
    members: T,
    size_hint: Option<usize>,
) -> Result<MapImpl<String, PklValue>> {
    let mut pkl_object = if let Some(size_hint) = size_hint {
        #[cfg(feature = "indexmap")]
        let map = IndexMap::with_capacity(size_hint);
        #[cfg(not(feature = "indexmap"))]
        let map = HashMap::with_capacity(size_hint);
        map
    } else {
        #[cfg(feature = "indexmap")]
        let map = IndexMap::new();
        #[cfg(not(feature = "indexmap"))]
        let map = HashMap::new();
        map
    };

    for member in members {
        let ObjectMember(k, v) = member;
        pkl_object.insert(k, v);
    }

    Ok(pkl_object)
}

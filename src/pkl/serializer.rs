use std::collections::BTreeMap;

use super::{
    internal::{ObjectMember, PklValue},
    PklMod,
};

pub trait PklSerialize {
    fn serialize_pkl_ast(self) -> anyhow::Result<BTreeMap<String, PklValue>>;
}

impl PklSerialize for Vec<ObjectMember> {
    fn serialize_pkl_ast(self) -> anyhow::Result<BTreeMap<String, PklValue>> {
        let mut pkl_object = BTreeMap::new();

        for member in self {
            let (k, v) = member.to_pkl_value()?;
            pkl_object.insert(k, v);
        }

        Ok(pkl_object)
    }
}

impl PklSerialize for PklMod {
    /// serializes the module into a btree map
    fn serialize_pkl_ast(self) -> anyhow::Result<BTreeMap<String, PklValue>> {
        let mut pkl_object = BTreeMap::new();

        for member in self.members {
            let (k, v) = member.to_pkl_value()?;
            pkl_object.insert(k, v);
        }

        Ok(pkl_object)
    }
}

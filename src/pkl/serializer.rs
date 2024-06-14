use std::collections::BTreeMap;

use super::{ObjectMember, PklMod, PklValue};

pub trait PklSerialize {
    // #[deprecated]
    // fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>>;

    fn serialize_pkl_ast(self) -> anyhow::Result<BTreeMap<String, PklValue>>;
}

impl PklSerialize for Vec<ObjectMember> {
    // fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    //     let mut json_object = serde_json::Map::new();

    //     for member in self.iter() {
    //         let (k, v) = member.to_pkl_value()?;
    //         json_object.insert(k, serde_json::to_value(v)?);
    //     }

    //     Ok(json_object)
    // }

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
    // fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    //     let mut json_object = serde_json::Map::new();
    //     // let mut obj = BTreeMap::new();

    //     for member in self.members.iter() {
    //         let (k, v) = member.to_pkl_value()?;
    //         // obj.insert(k, v);
    //         json_object.insert(k, serde_json::to_value(v)?);
    //         // json_object.insert(k, v);
    //     }

    //     Ok(json_object)
    // }

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

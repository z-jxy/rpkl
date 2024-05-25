use super::{ObjectMember, PklMod};

pub trait PklSerialize {
    fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>>;
}

impl PklSerialize for Vec<ObjectMember> {
    fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut json_object = serde_json::Map::new();

        for member in self.iter() {
            let (k, v) = member.to_json()?;
            json_object.insert(k, v);
        }

        Ok(json_object)
    }
}

impl PklSerialize for PklMod {
    fn serialize_json(&self) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut json_object = serde_json::Map::new();

        for member in self.members.iter() {
            let (k, v) = member.to_json()?;
            json_object.insert(k, v);
        }

        Ok(json_object)
    }
}

use std::path::PathBuf;

use crate::{api, pkl::PklSerialize};

pub trait PklLoadable {
    fn load_from_path(path: impl Into<PathBuf>) -> anyhow::Result<Self>
    where
        Self: Sized + for<'de> serde::Deserialize<'de>,
    {
        {
            let mut evaluator = api::Evaluator::new()?;
            let pkl_mod = evaluator.evaluate_module(path.into())?;

            let json = pkl_mod.serialize_json()?;
            let v: Self = serde_json::from_value(serde_json::Value::Object(json))?;
            Ok(v)
        }
    }
}

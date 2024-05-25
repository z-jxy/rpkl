use std::path::PathBuf;

use pkl::PklSerialize;

pub mod api;
pub mod pkl;
pub mod types;
// #[macro_export]
// macro_rules! include_pkl {
//     ($package: tt) => {
//         include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
//     };
// }

/// Evaluates a `.pkl` file and interprets it as `T`
///
/// `path`: The path to the `.pkl` file
///
/// # Example
///
/// ```rust
/// #[derive(Deserialize)]
/// struct Database {
///     username: String,
///     password: String,
/// }
///
/// let config: Database = pkl_rs::value_from_config("config.pkl")?;
/// ```
pub fn value_from_config<T>(path: impl Into<PathBuf>) -> anyhow::Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de>,
{
    {
        let mut evaluator = api::Evaluator::new()?;
        let pkl_mod = evaluator.evaluate_module(path.into())?;
        let json = pkl_mod.serialize_json()?;
        let v: T = serde_json::from_value(serde_json::Value::Object(json))?;
        Ok(v)
    }
}

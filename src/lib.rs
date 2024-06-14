use std::{fmt::Debug, io::Cursor};

use pkl::PklSerialize;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub mod api;
pub mod pkl;
pub mod types;
use api::error::{Error, Result};
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
/// ```pkl
/// ip = "127.0.0.1"
/// database {
///     username = "root"
///     password = "password"
/// }
/// ```
/// -------------
/// ```rust
///
/// #[derive(Deserialize)]
/// struct Config {
///     ip: String,
///     database: Database,
/// }
///
/// #[derive(Deserialize)]
/// struct Database {
///     username: String,
///     password: String,
/// }
///
/// let config: Database = pkl_rs::value_from_config("config.pkl")?;
/// ```
pub fn value_from_config<T>(path: impl AsRef<std::path::Path>) -> anyhow::Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de> + Debug,
{
    {
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::TRACE)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
        let mut evaluator = api::Evaluator::new()?;
        let mut pkl_mod = evaluator.evaluate_module(path.as_ref().to_path_buf())?;
        // let mut pkl_mod2 = evaluator.evaluate_module_as_slice(path.as_ref().to_path_buf())?;
        // let json = pkl_mod.serialize_json()?;
        let mut pkld = pkl_mod.serialize_pkl()?;

        println!("{:?}", pkld);
        // let mut buf_ = Vec::new();
        // let mut buf = Cursor::new(pkl_mod);

        // let valu = serde_json::Value::Object(serde_json::Map::new());
        let z = T::deserialize(&mut api::deserializer::Deserializer::from_pkl_map(
            &mut pkld,
        ));
        // if let Some(ip) = pkld.get("ip") {
        //     let s = ip.as_str().unwrap();
        //     println!("{}", s);
        //     // println!("{}", s[0..1].to_string());
        //     // println!("{}", ip.as_str().unwrap());
        // }
        println!("serde deserialized: {:?}", z);
        // if let Ok(db) = z {
        //     println!("sucess: {:?}", db);
        // }
        // let v: T = serde_json::from_value(serde_json::Value::Object(json))?;
        // let v = serde_
        z.map_err(|e| anyhow::anyhow!("failed to deserialize: {:?}", e))
    }
}

use std::fmt::Debug;

use api::deserializer::Deserializer;
use pkl::PklSerialize;

pub mod api;
pub mod pkl;
pub mod types;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

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
/// let config: Database = pkl_rs::from_config("config.pkl")?;
/// ```
pub fn from_config<T>(path: impl AsRef<std::path::Path>) -> anyhow::Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de> + Debug,
{
    {
        #[cfg(feature = "trace")]
        {
            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(Level::TRACE)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
        }

        let mut evaluator = api::Evaluator::new()?;
        let pkl_mod = evaluator.evaluate_module(path.as_ref().to_path_buf())?;

        let mut pkld = pkl_mod.serialize_pkl_ast()?;

        #[cfg(feature = "trace")]
        trace!("serialized pkl ast {:?}", pkld);

        T::deserialize(&mut Deserializer::from_pkl_map(&mut pkld))
            .map_err(|e| anyhow::anyhow!("failed to deserialize: {:?}", e))
    }
}

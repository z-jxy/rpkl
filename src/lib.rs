#![feature(duration_constructors)]

use pkl::Deserializer;
use pkl::PklSerialize;

pub mod api;
mod context;
pub mod error;
pub mod pkl;
mod utils;
pub mod value;

pub use error::{Error, Result};

pub use value::PklValue as Value;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

/// Evaluates a `.pkl` file and deserializes it as `T`.
///
/// `path`: The path to the `.pkl` file
///
/// # Example
///
/// ```ignore
/// ip = "127.0.0.1"
/// database {
///     username = "root"
///     password = "password"
/// }
/// ```
/// -------------
/// ```no_run
///
/// use serde::Deserialize;
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
/// # fn main() -> Result<(), rpkl::Error> {
/// let config: Database = rpkl::from_config("config.pkl")?;
/// #    Ok(())
/// # }
/// ```
pub fn from_config<T>(path: impl AsRef<std::path::Path>) -> Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de>,
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
        trace!("serialized pkl data {:?}", pkld);

        T::deserialize(&mut Deserializer::from_pkl_map(&mut pkld))
            .map_err(|e| Error::DeserializeError(format!("{}", e)))
    }
}

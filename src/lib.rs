use pkl::Deserializer;
use pkl::PklSerialize;

pub mod api;
mod context;
pub mod error;
mod internal;
pub mod pkl;
mod utils;
pub mod value;

pub use error::{Error, Result};

pub use api::evaluator::EvaluatorOptions;
pub use value::PklValue as Value;

#[cfg(feature = "trace")]
use tracing::{debug, error, span, trace, Level};
#[cfg(feature = "trace")]
use tracing_subscriber::FmtSubscriber;

/// Evaluates a `.pkl` file and deserializes it as `T`. If you need to pass options to the evaluator, such as properties, use [`from_config_with_options`].
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
/// let config: Config = rpkl::from_config("config.pkl")?;
/// #    Ok(())
/// # }
/// ```
pub fn from_config<T>(path: impl AsRef<std::path::Path>) -> Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de>,
{
    return from_config_with_options(path, EvaluatorOptions::default());
}

/// Allows for passing options to the evaluator, such as properties (e.g. `read("prop:username")`). See [`EvaluatorOptions`] for more information.
///
/// # Example
///
/// ```ignore
/// ip = "127.0.0.1"
/// credentials {
///     username = read("prop:username")
///     password = read("prop:password")
/// }
/// ```
/// -------------
/// ```no_run
///
/// use serde::Deserialize;
/// use rpkl::api::evaluator::EvaluatorOptions;
///
/// #[derive(Deserialize)]
/// struct Config {
///     ip: String,
///     database: Credentials,
/// }
///
/// #[derive(Deserialize)]
/// struct Credentials {
///     username: String,
///     password: String,
/// }
///
/// # fn main() -> Result<(), rpkl::Error> {
/// let options = EvaluatorOptions::default()
///     .properties([("username", "root"), ("password", "password123")]);
/// let config: Config = rpkl::from_config("config.pkl")?;
/// #    Ok(())
/// # }
/// ```
pub fn from_config_with_options<T>(
    path: impl AsRef<std::path::Path>,
    options: EvaluatorOptions,
) -> Result<T>
where
    T: Sized + for<'de> serde::Deserialize<'de>,
{
    #[cfg(feature = "trace")]
    {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    let mut evaluator = api::Evaluator::new_from_options(options)?;
    let pkl_mod = evaluator.evaluate_module(path.as_ref().to_path_buf())?;

    let mut pkld = pkl_mod.serialize_pkl_ast()?;

    #[cfg(feature = "trace")]
    trace!("serialized pkl data {:?}", pkld);

    T::deserialize(&mut Deserializer::from_pkl_map(&mut pkld))
        .map_err(|e| Error::DeserializeError(format!("{}", e)))
}

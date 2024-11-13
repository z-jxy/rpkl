use std::path::PathBuf;

use rpkl::api::evaluator::EvaluatorOptions;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    path: String,
    name: String,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("allowed-resources.pkl");

    let options = EvaluatorOptions::default().properties([("name", "Ferris")]);
    let config: Config = rpkl::from_config_with_options(path, Some(options)).unwrap();

    println!("{:?}", config);
}

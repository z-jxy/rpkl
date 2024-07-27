use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    string: Option<String>,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("options.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

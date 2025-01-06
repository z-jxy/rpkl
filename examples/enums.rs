use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    mode: Mode,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
enum Mode {
    Dev,
    Production,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("enums.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

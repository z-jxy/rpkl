use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    paths: HashMap<String, Vec<String>>,
    extensions: Vec<Vec<String>>,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("nested.pkl");
    let value = rpkl::from_config::<Config>(path);

    println!("{:?}", value);
}

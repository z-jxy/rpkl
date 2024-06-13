use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ip: String,
    database: Database,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Database {
    username: String,
    password: String,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("example.pkl");
    let value = pkl_rs::value_from_config::<Config>(path);
    // println!("{:?}", pkl_rs::value_from_config::<Config>(path));
}

use serde::Deserialize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ip: Option<String>,
    port: u16,
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
        .join("tests")
        .join("pkl")
        .join("example.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

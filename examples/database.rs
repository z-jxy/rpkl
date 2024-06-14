use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Config {
    ip: String,
    port: u16,
    birds: Vec<String>,
    database: Database,
    mapping: HashMap<String, String>,
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
    let value = pkl_rs::from_config::<Config>(path);
    println!("{:?}", value);
}

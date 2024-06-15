use pkl_rs::from_config;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Example {
    pub ip: String,
    pub port: i64,
    pub birds: Vec<pkl_rs::Value>,
    pub mapping: pkl_rs::Value,
    pub anon_map: example::AnonMap,
    pub database: example::Database,
}

mod example {
    use super::*;

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub(crate) struct AnonMap {
        pub anon_key: String,
    }

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    pub(crate) struct Database {
        pub username: String,
        pub password: String,
    }
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("example.pkl");
    let mut evaluator = pkl_rs::api::evaluator::Evaluator::new().unwrap();
    // let pkl_mod = evaluator.evaluate_module(path).unwrap();
    let x = from_config::<Example>(path);
    println!("{:#?}", x);
    // #[cfg(feature = "codegen")]
    // let _ = pkl_mod.codegen();

    // let value = pkl_rs::from_config::<Config>(path);
    // println!("{:?}", value);
}

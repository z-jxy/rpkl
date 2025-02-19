use rpkl::from_config;

use std::path::PathBuf;

/* Generated by rpkl */

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Example {
    pub ip: String,
    pub port: i64,
    pub birds: Vec<rpkl::Value>,
    pub mapping: rpkl::Value,
    pub anon_map: example::AnonMap,
    pub database: example::Database,
}

pub mod example {
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct AnonMap {
        pub anon_key: String,
        #[serde(rename = "anon_key2")]
        pub anon_key_2: String,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Database {
        pub username: String,
        pub password: String,
    }
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("example.pkl");

    #[cfg(feature = "codegen")]
    {
        let mut evaluator = rpkl::api::evaluator::Evaluator::new().unwrap();
        let pkl_mod = evaluator.evaluate_module(path).unwrap();
        let _ = pkl_mod.codegen();
        return;
    }

    println!("{:?}", from_config::<Example>(&path));
}

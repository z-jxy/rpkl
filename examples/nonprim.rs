use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
#[allow(dead_code)]
pub struct Config {
    duration: std::time::Duration,
    size: rpkl::DataSize,
    pair: (i32, i32),
    range: std::ops::Range<i64>,
    #[serde(rename(deserialize = "emailRegex"))]
    email_regex: String,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("nonprim.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

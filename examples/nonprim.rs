use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    duration: std::time::Duration,
    size: rpkl::DataSize,
    pair: (i32, i32),
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("nonprim.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

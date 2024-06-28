use std::path::PathBuf;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    duration: std::time::Duration,
}

fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("nonprim.pkl");
    let value = rpkl::from_config::<Config>(path);
    println!("{:?}", value);
}

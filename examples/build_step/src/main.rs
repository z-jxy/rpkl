use rpkl::from_config;

#[allow(dead_code)]
mod pkl {
    include!(concat!(env!("OUT_DIR"), concat!("/mod.rs")));
}

fn main() {
    from_config::<pkl::Example>("../../tests/pkl/example.pkl").unwrap();
    from_config::<pkl::Database>("../../tests/pkl/database.pkl").unwrap();
}

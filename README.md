# rpkl

[<img alt="crates.io" src="https://img.shields.io/crates/v/rpkl?style=for-the-badge&color=fc8d62&logo=rust" height="20" />](https://crates.io/crates/rpkl)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-rpkl-6b9543?style=for-the-badge&logo=docs.rs&labelColor=555555" height="20">](https://docs.rs/rpkl)

Language bindings to Pkl for Rust.

Requires the pkl binary to be available on your path. You can install pkl for your os using the steps from their docs: <https://pkl-lang.org/main/current/pkl-cli/index.html#installation>

## Usage

```pkl
ip = "127.0.0.1"

database {
    username = "admin"
    password = "secret"
}
```

```rust
#[derive(Deserialize)]
struct Config {
    ip: String,
    database: Database,
}

#[derive(Deserialize)]
struct Database {
    username: String,
    password: String,
}

let config: Config = rpkl::value_from_config("./config.pkl")?;
```

## Codegen

Mostly works, but still a WIP. If you want to try it out, you can enable the `codegen` feature.

```rust
let mut evaluator = rpkl::evaluator::Evaluator::new()?;
let pkl_mod = evaluator.evaluate_module(PathBuf::from("./config.pkl"))?;
pkl_mod.codegen()?;
```

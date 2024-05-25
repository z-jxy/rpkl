# pkl-rs

Language bindings to Pkl for Rust.

Requires the pkl binary to be available on your path. You can install pkl for your os using the steps from their docs: <https://pkl-lang.org/main/current/pkl-cli/index.html#installation>

## Usage

```rust
#[derive(Deserialize)]
struct Database {
    username: String,
    password: String,
}

let config: Database = pkl_rs::value_from_config("./config.pkl")?;
```

## Codegen

Mostly works, but still a WIP. If you want to try it out you can use the following snippet of code

```rust
let mut evaluator = pkl_rs::evaluator::Evaluator::new()?;
let pkl_mod = evaluator.evaluate_module(PathBuf::from("./config.pkl"))?;
pkl_mod.codegen()?;
```

This can be added to a `build.rs` file to run at build time to generate structs.

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

let config: Config = rpkl::from_config("./config.pkl")?;
```

### Evaluator Options

You can pass options to the evaluator, such as properties, by using [`from_config_with_options`].

```pkl
username = read("prop:username")
password = read("prop:password")
```

```rust
let options = EvaluatorOptions::default()
.properties([("username", "root"), ("password", "password123")]);

let config: Config = rpkl::from_config_with_options("./config.pkl", Some(options))?;
```

## Codegen

Codegen is still being improved, but should work for most general cases. If you want to try it out, you can enable the `codegen` feature.

```rust
use rpkl::{api::Evaluator, codegen::CodegenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut evaluator = Evaluator::new()?;
    let pkl_mod = evaluator.evaluate_module("example.pkl")?;
    let options = CodegenOptions::default();
    let code: String = pkl_mod.codegen(Some(options))?;
    std::fs::write("src/example.rs", code)?;
    Ok(())
}
```

If you want to run codegen as part of a build script, consider using the `build-script` feature

```rust
use rpkl::EvaluatorOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rpkl::build_script::configure()
        .as_enum("Example.mode", &["Dev", "Production"])
        .opaque("Example.mapping")
        .evaluator_options(EvaluatorOptions::default())
        .output("generated/mod.rs") // save to custom output file
        .codegen(&[
            "../../tests/pkl/example.pkl",
            "../../tests/pkl/database.pkl",
        ])
}
```

# Codegen

Rpkl codegen works by generating a struct for the pkl module you pass it, along with a rust module containing any structs the main struct depends on. For instance, generating code for the following module:

```pkl
ip = "127.0.0.1"
port = 8080

birds: Listing<String> = new {
  "Pigeon"
  "Hawk"
  "Penguin"
}

anon_map = new {
  ["anon_key"] = "anon_value"
  ["anon_key2"] = "anon_value2"
}

mode: "Dev" | "Production" = "Dev"
```

Results in the following code:

```rust
#[derive(Debug, ::serde::Deserialize)]
pub struct Example {
    pub ip: String,
    pub port: i64,
    pub birds: Vec<rpkl::Value>,
    pub anon_map: example::AnonMap,
    pub mode: String,
}

pub mod example {
    #[derive(Debug, ::serde::Deserialize)]
    pub struct AnonMap {
        #[serde(rename = "anon_key2")]
        pub anon_key_2: String,
        pub anon_key: String,
    }
}
```

## Modifying Generated Output

It's possible to modify the generated code by configuring the `CodegenOptions`

> [!NOTE]
Settings for modifying codegen are still experimental.
Expect either the API, or its underlying behavior to change in the future.

### Type Attributes

You can add custom attributes to generated struct types:

```rust
.type_attribute("Example", "#[derive(Clone)]")
```

This adds `#[derive(Clone)]` to the generated `Example` struct.

To target structs within the generated module (like `AnonMap` above) you can prefix the struct name with the name of the module.

In the above example, the generated module would be `config`, so you would specify the target as `config.AnonMap`.

### Enum Generation

Pkl doesn't directly support enums, but can enforce constraints on string values like:

```pkl
mode: "Dev" | "Production"
```

When evaluating a module, pkl will only send information about the string value, not the contraints, so ultimately the field will be generated as a `String`. You can tell the codegen to instead treat this field as an enum:

```rust
.as_enum("Config.mode", &["Dev", "Production"])
```

This will generate:

```rust
#[derive(Debug, ::serde::Deserialize)]
pub struct Config {
    pub mode: Mode,
    // other fields...
}

#[derive(Debug, ::serde::Deserialize)]
pub enum Mode {
    Dev,
    Production,
}
```

### Field Attributes

Add attributes to specific fields in your generated structs:

```rust
.field_attribute("Mode.Dev", "#[default]")
```

This adds `#[default]` to the `Dev` variant in the `Mode` enum.

### Opaque Type Fields

By default, maps/mappings will be generated as a struct with all fields found during the first evaluation. If you instead want to treat the field as a generic ([PklValue](https://docs.rs/rpkl/latest/rpkl/value/value/enum.PklValue.html)) you can use the `--opaque` option:

```rust
.opaque("Config.dynamicSettings")
```

This will generate the field as `pub dynamic_config: rpkl::Value`

# rpkl-cli

CLI tool for generating Rust code from Pkl configuration files.

## Overview

`rpkl-cli` generates Rust structs from Pkl configuration files. It also offers various customization options to control how the generated code is structured, including adding custom attributes to types and fields, generating enums from string literals, and more.

## Requirements

Requires Pkl to be installed on your system. See [Pkl's](https://pkl-lang.org/main/current/pkl-cli/index.html#installation) installation guide.

## Command Line Options

| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Specify an output file (defaults to stdout) |
| `--type-attribute <STRUCT=ATTR>` | Add attributes to generated structs (format: `MyStruct=#[derive(Default)]`) |
| `--field-attribute <STRUCT.FIELD=ATTR>` | Add attributes to generated struct fields (format: `MyStruct.fieldName=#[default]`) |
| `--as-enum <STRUCT.FIELD=VARIANT1,VARIANT2>` | Generate an enum for string fields (format: `MyStruct.fieldName=Variant1,Variant2`) |
| `--opaque <STRUCT.FIELD>` | Use opaque type (`rpkl::Value`) for specified field (format: `MyStruct.fieldName`) |

## Usage

Basic usage:

```bash
rpkl path/to/your/config.pkl
```

This will evaluate the Pkl file and output the generated Rust code to stdout.

## Code Generation Features

### Type Attributes

You can add custom attributes to generated struct types:

```bash
rpkl --type-attribute "Config=#[derive(Default)]" input.pkl
```

This adds `#[derive(Default)]` to the generated `Config` struct.

### Field Attributes

Add attributes to specific fields in your generated structs:

```bash
rpkl --field-attribute "Mode.Dev=#[default]" input.pkl
```

This adds a `#[default]` to the `Dev` variant in the `Mode` enum.

### Enum Generation

Pkl doesn't directly support enums, but can enforce constraints on string values like:

```pkl
mode: "Dev" | "Production"
```

When evaluating a module, pkl will only send information about the string value, not the contraints, so ultimately the field will be generated as a `String`. You can tell the codegen to instead treat this field as an enum:

```bash
rpkl --as-enum "Config.mode=Dev,Production" input.pkl
```

This will generate:

```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub mode: Mode,
    // other fields...
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum Mode {
    Dev,
    Production,
}
```

You can also add attributes to the generated enum and its variants:

```bash
rpkl \
  --as-enum "Config.mode=Dev,Production" \
  --type-attribute "Mode=#[derive(Default)]" \
  --field-attribute "Mode.Dev=#[default]" \
  input.pkl
```

### Opaque Type Fields

By default, maps/mappings will be generated as a struct with all fields found during the first evaluation. If you instead want to treat the field as a generic ([PklValue](https://docs.rs/rpkl/latest/rpkl/value/value/enum.PklValue.html)) you can use the `--opaque` option:

```bash
rpkl --opaque "Config.dynamicSettings" input.pkl
```

This will generate the field as `pub dynamic_config: rpkl::Value`

## License

This project is licensed under the MIT License.

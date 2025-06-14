use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use rpkl::{api::Evaluator, codegen::CodegenOptions};

fn main() {
    let cli = Cli::parse();
    let mut evaluator = match Evaluator::new() {
        Ok(evaluator) => evaluator,
        Err(e) => {
            eprintln!("Failed to create evaluator: {e}");
            std::process::exit(1);
        }
    };

    let pkl_mod = match evaluator.evaluate_module(cli.file) {
        Ok(pkl_mod) => pkl_mod,
        Err(e) => {
            eprintln!("Failed to evaluate module: {e}");
            std::process::exit(1);
        }
    };

    let mut options = CodegenOptions::default();
    for mapping in cli.type_attribute {
        options = options.type_attribute(mapping.ident, mapping.value);
    }
    for mapping in cli.field_attribute {
        options = options.field_attribute(mapping.ident, mapping.value);
    }
    for mapping in cli.as_enum {
        options = options.as_enum(mapping.ident, &mapping.variants);
    }
    for mapping in cli.opaque {
        options = options.opaque(mapping);
    }

    let code = match pkl_mod.codegen_with_options(options) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to generate code: {e}");
            std::process::exit(1);
        }
    };

    match cli.output {
        Some(output) => {
            if let Some(parent) = output.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create output directory: {e}");
                    std::process::exit(1);
                }
            }

            if let Err(e) = std::fs::write(output, code) {
                eprintln!("Failed to write generated code to file: {e}");
                std::process::exit(1);
            };
        }
        None => println!("{code}"),
    }
}

/// CLI for the rpkl code generator
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
    /// The output file to write the generated code to, if not specified, the code will be printed to stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// The pkl file to generate code for
    file: PathBuf,

    /// Mapping of type attributes to values to add to a generated struct
    /// Ex: `Example=#[derive(Default)]`, `Mode=#[derive(Default)]`
    #[arg(long, value_name = "STRUCT=ATTR")]
    type_attribute: Vec<ValueMapping>,

    /// Mapping of field attributes to apply to a generated struct fields
    /// Ex: `Mode.Dev=#[default]`
    #[arg(long, value_name = "STRUCT.FIELD=ATTR")]
    field_attribute: Vec<ValueMapping>,

    /// Forces the generated code to use an enum for the specified field
    /// Ex: `Example.mode=Dev,Production`
    #[arg(long, value_name = "STRUCT.FIELD=VARIANT1,VARIANT2")]
    as_enum: Vec<EnumVariantMapping>,

    /// Forces the generated code to use an opaque type for the specified field (rpkl::Value)
    /// Ex: `Example.mapping`
    #[arg(long, value_name = "STRUCT.FIELD")]
    opaque: Vec<String>,
}

#[derive(Debug, Clone)]
struct ValueMapping {
    ident: String,
    value: String,
}

impl FromStr for ValueMapping {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((ident, value)) = s.split_once('=') else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid value mapping, expected an identifier and a value separated by `=`",
            ));
        };

        Ok(ValueMapping {
            ident: ident.to_string(),
            value: value.to_string(),
        })
    }
}

// same as value mapping, but expects one or more values after the `=`, separated by commas
#[derive(Debug, Clone)]
struct EnumVariantMapping {
    ident: String,
    variants: Vec<String>,
}

impl FromStr for EnumVariantMapping {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((ident, value)) = s.split_once('=') else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid value mapping",
            ));
        };
        Ok(EnumVariantMapping {
            ident: ident.to_string(),
            variants: value.split(',').map(|s| s.trim().to_string()).collect(),
        })
    }
}

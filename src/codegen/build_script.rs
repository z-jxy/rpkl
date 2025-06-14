//!
//! Limitations:
//!
//! - When generating code from multiple pkl files, the module names must be unique.
//! Currently, rpkl generates a single `mod.rs` file with all the generated code.
//!
//!

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::{EvaluatorOptions, api::Evaluator};

use super::{CODEGEN_HEADER, CodegenOptions};

pub fn configure() -> Builder {
    Builder::default()
}

impl Builder {
    pub fn codegen(self, modules: &[impl AsRef<Path>]) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.output.unwrap_or(
            PathBuf::from(std::env::var("OUT_DIR").expect(
                "OUT_DIR not set, expected to be run in a build script or have an output path set",
            ))
            .join("mod.rs"),
        );

        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        };

        let mut evaluator = Evaluator::new_from_options(self.evaluator_options)?;

        let mut output_file = std::fs::File::create(output_path)?;
        output_file.write_all(CODEGEN_HEADER.as_bytes())?;

        for module in modules {
            let module_path = module.as_ref();
            if self.rerun_if_changed {
                println!("cargo:rerun-if-changed={}", module_path.display());
            }

            let pkl_mod = evaluator.evaluate_module(module_path)?;
            let code = pkl_mod.codegen_with_options(&self.codegen_options)?;

            output_file.write_all(&code.as_bytes()[CODEGEN_HEADER.len()..])?;
        }

        Ok(())
    }
}

pub struct Builder {
    codegen_options: CodegenOptions,
    evaluator_options: EvaluatorOptions,
    output: Option<PathBuf>,
    rerun_if_changed: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            codegen_options: CodegenOptions::default(),
            evaluator_options: EvaluatorOptions::default(),
            output: None,
            rerun_if_changed: true,
        }
    }
}

impl Builder {
    /// Mapping of type attributes to values to add to a generated struct
    ///
    /// Ex: `Example=#[derive(Default)]`, `Mode=#[derive(Default)]`
    pub fn type_attribute(mut self, ident: &str, value: &str) -> Self {
        self.codegen_options = self.codegen_options.type_attribute(ident, value);
        self
    }

    /// Mapping of field attributes to apply to a generated struct fields
    ///
    /// Ex: `Mode.Dev=#[default]`
    pub fn field_attribute(mut self, ident: &str, value: &str) -> Self {
        self.codegen_options = self.codegen_options.field_attribute(ident, value);
        self
    }

    /// Forces the generated code to use an enum for the specified field
    ///
    /// Ex: `Example.mode=Dev,Production`
    pub fn as_enum(mut self, ident: &str, variants: &[&str]) -> Self {
        self.codegen_options = self.codegen_options.as_enum(ident, variants);
        self
    }

    /// Use an opaque type for the specified field (rpkl::Value)
    ///
    /// Ex: `Example.mapping`
    pub fn opaque(mut self, ident: &str) -> Self {
        self.codegen_options = self.codegen_options.opaque(ident);
        self
    }

    /// Pass in [`EvaluatorOptions`] to configure the evaluator.
    pub fn evaluator_options(mut self, options: EvaluatorOptions) -> Self {
        self.evaluator_options = options;
        self
    }

    /// Sets the output path for the generated code. If not set, the code will be written to `OUT_DIR`.
    ///
    /// <https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates>
    pub fn output(mut self, output: impl AsRef<Path>) -> Self {
        self.output = Some(output.as_ref().to_path_buf());
        self
    }

    /// When set to `true`, the build script will rerun when any of the input files change.
    /// Default is `true`.
    ///
    /// If you'd rather control this with the `cargo:rerun-if-changed` directive, set this to `false`.
    pub fn rerun_if_changed(mut self, rerun_if_changed: bool) -> Self {
        self.rerun_if_changed = rerun_if_changed;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::pkl_tests_file;

    #[test]
    #[should_panic]
    fn test_codegen() {
        let path = pkl_tests_file("example.pkl");
        let output_path = pkl_tests_file("example.rs");

        configure()
            .type_attribute("Example", "#[derive(Default)]")
            .field_attribute("Example.ip", "#[serde(rename = \"ip\")]")
            .as_enum("Example.mode", &["Dev", "Production"])
            .type_attribute("Mode", "#[derive(Default)]")
            .field_attribute("Mode.Dev", "#[default]")
            .opaque("Example.mapping")
            .output(output_path)
            .codegen(&[path])
            .unwrap();
    }
}

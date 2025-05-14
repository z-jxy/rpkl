use crate::internal::ObjectMember;

#[derive(Debug)]
pub struct PklMod {
    pub(crate) module_name: String,
    pub(crate) module_uri: String,
    pub(crate) members: Vec<ObjectMember>,
}

impl PklMod {
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn module_uri(&self) -> &str {
        &self.module_uri
    }
}

#[cfg(feature = "codegen")]
pub mod codegen {
    use convert_case::{Case, Casing};
    use std::collections::HashSet;

    use std::fmt::Write as _;

    use crate::internal::{Integer, ObjectMember};

    use super::PklMod;
    use crate::{Result, Value as PklValue};

    /// Config to modify the code generated from [`PklMod::codegen`].
    #[derive(Default)]
    pub struct CodegenOptions {
        type_attributes: Vec<(String, String)>,
        field_attributes: Vec<(String, String)>,
        enums: Vec<(String, String)>,
        infer_vec_types: bool,
        opaque_fields: HashSet<String>,
    }

    impl CodegenOptions {
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        /// Add addtional attributes to the matched struct
        ///
        /// # Examples
        ///
        /// This will add `#[derive(Default)]` to the generated struct `MyStruct`.
        ///
        /// ```rust
        /// use rpkl::codegen::CodegenOptions;
        /// let options = CodegenOptions::new()
        ///    .type_attribute("MyStruct", "#[derive(Default)]");
        /// ``````
        pub fn type_attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.type_attributes.push((name.into(), value.into()));
            self
        }

        /// Add addtional attributes to the matched field
        ///
        /// # Examples
        ///
        /// This will add `#[serde(rename = "ip")]` to the generated field `ip` in the struct `Example`.
        ///
        /// ```rust
        /// use rpkl::codegen::CodegenOptions;
        /// let options = CodegenOptions::new()
        ///    .field_attribute("Example.ip", "#[serde(rename = \"ip\")]");
        /// ```
        pub fn field_attribute(
            mut self,
            name: impl Into<String>,
            value: impl Into<String>,
        ) -> Self {
            self.field_attributes.push((name.into(), value.into()));
            self
        }

        /// Forces a field to be generated as an enum.
        ///
        /// Pkl doesn't directly support enums, but it's possible to have a string be validated aginst a set of values
        /// like so:
        ///
        /// ```pkl
        /// mode: "Dev" | "Production"
        /// ```
        ///
        /// When amending to a module with a member defined like this, pkl will validate the value during evaluation,
        /// however it's ultimately just a string. This results in the generated field being a string.
        ///
        /// This method will generate an enum for the field, and add the variants to the generated code.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use rpkl::codegen::CodegenOptions;
        /// let options = CodegenOptions::new()
        ///    .as_enum("Example.mode", &["Dev", "Production"]);
        /// ```
        ///
        /// The enum and it's variants can also be targeted for modifications using
        /// [`CodegenOptions::type_attribute`] and [`CodegenOptions::field_attribute`] methods:
        ///
        /// ```rust
        /// use rpkl::codegen::CodegenOptions;
        /// let options = CodegenOptions::new()
        ///    .as_enum("Example.mode", &["Dev", "Production"])
        ///   .type_attribute("Mode", "#[derive(Default)]")
        ///   .field_attribute("Mode.Dev", "#[default]");
        /// ```
        pub fn as_enum(mut self, name: impl Into<String>, variants: &[&str]) -> Self {
            self.enums.push((name.into(), variants.join(",\n")));
            self
        }

        /// When set to `true`, the codegen will try to infer the type of lists and sets
        /// based on the values in the list/set.
        ///
        /// If the values are all the same type, it will generate a `Vec<T>` instead of `Vec<rpkl::Value>`.
        /// If the values are different types, it will fallback to the default of `Vec<rpkl::Value>`.
        pub fn infer_vec_types(mut self, infer: bool) -> Self {
            self.infer_vec_types = infer;
            self
        }

        /// Forces a field type to be generated as an opaque value (rpkl::Value).
        pub fn opaque(mut self, name: impl Into<String>) -> Self {
            self.opaque_fields.insert(name.into());
            self
        }

        fn find_type_attribute(&self, name: &str) -> Option<&String> {
            self.type_attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v)
        }

        fn find_field_attribute(&self, name: &str) -> Option<&String> {
            self.field_attributes
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v)
        }

        fn find_enum(&self, name: &str) -> Option<&String> {
            self.enums.iter().find(|(n, _)| n == name).map(|(_, v)| v)
        }

        fn is_forced_opaque(&self, name: &str) -> bool {
            self.opaque_fields.contains(name)
        }
    }

    impl PklMod {
        /// By default, all structs are generated with `Debug`, `serde::Deserialize` and `serde::Serialize` attributes.
        ///
        /// To modify the generated code,
        /// use [`CodegenOptions`] to add additional attributes to the generated structs and fields.
        ///
        /// # Errors
        /// Errors if the generated code cannot be written to the file system.
        // TODO: return result as a string
        pub fn codegen(&self, options: Option<CodegenOptions>) -> Result<String> {
            let options = options.unwrap_or_default();
            // let path = PathBuf::from("./generated");
            // std::fs::create_dir_all(&path)?;
            // let file = std::fs::File::create(path.join("mod.rs"))?;

            let module_name = &self.module_name;

            // let mut writer = std::io::BufWriter::new(file);
            let mut writer = String::new();

            writeln!(writer, "/* Generated by rpkl */\n")?;

            let mut generated_structs = HashSet::new();

            let context = Context { options };
            let (code, deps) = context.generate_struct(
                module_name,
                &self.members,
                false,
                module_name,
                true,
                &mut generated_structs,
            )?;

            writeln!(writer, "{code}")?;

            if !deps.is_empty() {
                writeln!(
                    writer,
                    "pub mod {} {{",
                    self.module_name.to_case(Case::Snake)
                )?;
                // TODO: reimplement
                // writeln!(writer, "\tuse super::*;\n")?;
                for dep in &deps {
                    for line in dep.lines() {
                        writeln!(writer, "\t{line}")?;
                    }
                }

                writeln!(writer, "}}")?;
            }

            Ok(writer)
        }
    }

    struct Context {
        options: CodegenOptions,
    }

    impl Context {
        fn field_type_from_pkl_value(&self, value: &PklValue) -> String {
            match value {
                PklValue::Boolean(_) => "bool".to_string(),
                PklValue::Int(integer) => match integer {
                    Integer::Pos(_) => "u64".to_string(),
                    Integer::Neg(_) => "i64".to_string(),
                    Integer::Float(_) => "f64".to_string(),
                },
                PklValue::String(_) => "String".to_string(),
                PklValue::Null => "Option<rpkl::Value>".to_string(),
                PklValue::Map(_) => "rpkl::Value".to_string(),
                PklValue::List(values) => {
                    if self.options.infer_vec_types {
                        format!(
                            "Vec<{}>",
                            self.try_infer_list_type(values)
                                .unwrap_or("rpkl::Value".into())
                        )
                    } else {
                        "Vec<rpkl::Value>".to_string()
                    }
                }
                PklValue::Range(_) => "std::ops::Range<i64>".to_string(),
                PklValue::DataSize(_)
                | PklValue::Duration(_)
                | PklValue::Pair(_, _)
                | PklValue::Regex(_) => "rpkl::Value".to_string(),
            }
        }

        fn try_infer_list_type(&self, values: &[PklValue]) -> Option<String> {
            if values.is_empty() {
                return None;
            }

            if values.len() == 1 {
                return Some(self.field_type_from_pkl_value(&values[0]));
            }

            let mut types: HashSet<String> =
                HashSet::from([self.field_type_from_pkl_value(&values[0])]);

            // if we're able to insert any new type into the set,
            // we have multiple different types and cannot infer the the value of the vec
            // fall back to `Vec<rpkl::Value>`
            if values[1..]
                .iter()
                .any(|v| types.insert(self.field_type_from_pkl_value(v)))
            {
                return None;
            }

            assert!(types.len() == 1);
            Some(types.into_iter().next().unwrap())
        }

        fn generate_field(
            &self,
            member: &ObjectMember,
            (snake_case_field_name, top_level_module_name): (&str, &str),
            deps: &mut Vec<String>,
            generated_structs: &mut HashSet<String>,
            parent_struct_ident: &str,
        ) -> Result<String> {
            let mut field = String::new();
            let ObjectMember(_, member_ident, member_value) = member;

            // generate as an opaque value or generate the full struct?
            // downside of generating the full struct is that if a user wanted to be able to reload a configuration
            // if the value of a typed dynamic struct changes, it wont be able to pick up the new values
            // best approach is probably to let the user specify in a field should just be an opaque value

            let field_modifier = format!("{parent_struct_ident}.{snake_case_field_name}");

            // if let IPklValue::NonPrimitive(PklNonPrimitive::TypedDynamic(
            //     _,
            //     _,
            //     _,
            //     dynamic_members,
            // )) = member_value
            let is_forced_opaque = self.options.is_forced_opaque(&field_modifier);
            if let PklValue::Map(dynamic_members) = member_value {
                // generate the struct if they didn't specify for it to be opaque
                if !is_forced_opaque {
                    let (dep, child_deps) = self.generate_struct(
                        member_ident,
                        dynamic_members
                            .iter()
                            .map(|(k, v)| ObjectMember(0xFF, k.clone(), v.clone()))
                            .collect::<Vec<_>>()
                            .as_slice(),
                        // dynamic_members,
                        true,
                        top_level_module_name,
                        false,
                        generated_structs,
                    )?;
                    deps.push(dep);
                    deps.extend(child_deps);

                    // add the field
                    let upper_camel = member_ident.to_case(Case::UpperCamel);
                    let rename = if *member_ident == upper_camel {
                        Some(format!("#[serde(rename = \"{upper_camel}\")]\n",))
                    } else {
                        None
                    };

                    if let Some(rename) = &rename {
                        write!(field, "\t{rename}")?;
                    }
                    writeln!(
                        field,
                        "\tpub {snake_case_field_name}: {}::{upper_camel},",
                        top_level_module_name.to_case(Case::Snake),
                    )?;

                    return Ok(field);
                }
            }

            if let Some(attr) = self.options.find_enum(&field_modifier) {
                let variants = attr.split(',').map(str::trim).collect::<Vec<_>>();
                let __enum = self.generate_enum(member_ident, &variants, true, generated_structs);
                deps.push(__enum);
                writeln!(
                    field,
                    "\tpub {snake_case_field_name}: {}::{},",
                    top_level_module_name.to_case(Case::Snake),
                    member_ident.to_case(Case::UpperCamel),
                )?;
                return Ok(field);
            }

            if let Some(attr) = self.options.find_field_attribute(&field_modifier) {
                writeln!(field, "\t{attr}")?;
            }

            let rename = if snake_case_field_name == member_ident {
                None
            } else {
                Some(format!("#[serde(rename = \"{member_ident}\")]\n"))
            };

            if let Some(rename) = &rename {
                write!(field, "\t{rename}")?;
            }

            let field_type = if self.options.is_forced_opaque(&field_modifier) {
                "rpkl::Value"
            } else {
                &self.field_type_from_pkl_value(member_value)
            };

            writeln!(
                field,
                "\tpub {snake_case_field_name}: {field_type},",
                // &self.field_type_from_ipkl_value(&member_value)
                // &self.field_type_from_pkl_value(member_value)
            )?;

            Ok(field)
        }

        fn generate_enum(
            &self,
            enum_ident: &str,
            variants: &[&str],
            is_dependency: bool,
            generated_structs: &mut HashSet<String>,
        ) -> String {
            let upper_camel = enum_ident.to_case(Case::UpperCamel);
            if generated_structs.contains(&upper_camel) {
                return String::new();
            }
            generated_structs.insert(upper_camel.clone());

            let mut code = String::new();

            if let Some(attr) = self.options.find_type_attribute(&upper_camel) {
                _ = writeln!(code, "{attr}");
            }

            code.push_str("#[derive(Debug, serde::Deserialize, serde::Serialize)]\n");

            if is_dependency {
                // code.push_str(&format!("pub(crate) struct {upper_camel} {{\n")); // TODO: revisit this
                _ = writeln!(code, "pub enum {upper_camel} {{");
            } else {
                _ = writeln!(code, "pub enum {upper_camel} {{");
            }

            for variant in variants {
                let variant_ident = variant.to_case(Case::UpperCamel);
                let varient_modifier_key = format!("{upper_camel}.{variant_ident}");
                if let Some(attrs) = self
                    .options
                    .field_attributes
                    .iter()
                    .find(|(name, _)| *name == varient_modifier_key)
                {
                    _ = writeln!(code, "\t{}", attrs.1);
                }
                _ = writeln!(code, "\t{variant_ident},");
            }

            code.push_str("}\n\n");

            code
        }

        fn generate_struct(
            &self,
            struct_ident: &str,
            members: &[ObjectMember],
            is_dependency: bool,
            top_level_module_name: &str,
            pub_struct: bool,
            generated_structs: &mut HashSet<String>,
        ) -> Result<(String, Vec<String>)> {
            let upper_camel = struct_ident.to_case(Case::UpperCamel);
            if generated_structs.contains(&upper_camel) {
                return Ok((String::new(), vec![]));
            }
            generated_structs.insert(upper_camel.clone());

            let mut code = String::new();

            if let Some(attr) = self.options.find_type_attribute(&upper_camel) {
                _ = writeln!(code, "{attr}");
            }
            code.push_str("#[derive(Debug, serde::Deserialize, serde::Serialize)]\n");

            if is_dependency {
                // code.push_str(&format!("pub(crate) struct {upper_camel} {{\n")); // TODO: revisit this
                _ = writeln!(code, "pub struct {upper_camel} {{");
            } else if pub_struct {
                _ = writeln!(code, "pub struct {upper_camel} {{");
            } else {
                _ = writeln!(code, "struct {upper_camel} {{");
            }

            // code.push_str(&format!("struct {} {{\n", struct_ident));

            let mut deps = vec![];
            for member in members {
                let member_ident = member.get_ident();
                // let member_value = member.get_value();
                let snake_case_field_name = member_ident.to_case(Case::Snake);
                let field = self.generate_field(
                    // (member_ident, member_value),
                    member,
                    (&snake_case_field_name, top_level_module_name),
                    &mut deps,
                    generated_structs,
                    &upper_camel,
                )?;
                code.push_str(&field);
            }

            code.push_str("}\n\n");

            Ok((code, deps))
        }
    }

    impl From<std::fmt::Error> for crate::Error {
        fn from(e: std::fmt::Error) -> Self {
            crate::Error::Message(format!("failed to write generated code: {e:?}"))
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::utils::tests::pkl_tests_file;

        use super::*;

        /// this test relies on iterating over members in the same order as the pkl file
        #[cfg(feature = "indexmap")]
        #[test]
        fn test_codegen_indexmap() {
            let expected = r#"/* Generated by rpkl */

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Example {
	#[serde(rename = "ip")]
	pub ip: String,
	pub port: i64,
	pub ints: std::ops::Range<i64>,
	pub birds: Vec<rpkl::Value>,
	pub mapping: rpkl::Value,
	pub anon_map: example::AnonMap,
	pub database: example::Database,
	pub mode: example::Mode,
}


pub mod example {
	#[derive(Default)]
	#[derive(Debug, serde::Deserialize, serde::Serialize)]
	pub struct AnonMap {
		pub anon_key: String,
		#[serde(rename = "anon_key2")]
		pub anon_key_2: String,
	}
	
	#[derive(Debug, serde::Deserialize, serde::Serialize)]
	pub struct Database {
		pub username: String,
		pub password: String,
	}
	
	#[derive(Default)]
	#[derive(Debug, serde::Deserialize, serde::Serialize)]
	pub enum Mode {
		#[default]
		Dev,
		Production,
	}
	
}
"#;

            let path = pkl_tests_file("example.pkl");

            let mut evaluator = crate::api::evaluator::Evaluator::new().unwrap();
            let pkl_mod = evaluator.evaluate_module(path).unwrap();
            let options = crate::codegen::CodegenOptions::default()
                .type_attribute("AnonMap", "#[derive(Default)]")
                .field_attribute("Example.ip", "#[serde(rename = \"ip\")]")
                .as_enum("Example.mode", &["Dev", "Production"])
                .type_attribute("Mode", "#[derive(Default)]")
                .field_attribute("Mode.Dev", "#[default]")
                .opaque("Example.mapping");
            let _ = pkl_mod.codegen(Some(options));

            let contents = std::fs::read_to_string("./generated/mod.rs").unwrap();
            assert_eq!(expected, contents);
        }

        #[test]
        fn test_codegen() {
            let path = pkl_tests_file("example.pkl");

            let mut evaluator = crate::api::evaluator::Evaluator::new().unwrap();
            let pkl_mod = evaluator.evaluate_module(path).unwrap();
            let options = crate::codegen::CodegenOptions::default()
                .type_attribute("AnonMap", "#[derive(Default)]")
                .field_attribute("Example.ip", "#[serde(rename = \"ip\")]")
                .as_enum("Example.mode", &["Dev", "Production"])
                .type_attribute("Mode", "#[derive(Default)]")
                .field_attribute("Mode.Dev", "#[default]")
                .opaque("Example.mapping");
            let contents = pkl_mod.codegen(Some(options)).unwrap();

            // check that the file contains all required struct and enum declarations
            assert!(contents.contains("pub struct Example"));
            assert!(contents.contains("pub struct AnonMap"));
            assert!(contents.contains("pub struct Database"));
            assert!(contents.contains("pub enum Mode"));

            // check for proper module structure
            assert!(contents.contains("pub mod example {"));

            // check for proper type attributes
            assert!(contents.contains("#[derive(Default)]"));
            assert!(contents.contains("#[default]"));

            // check for specific field presence using regex
            let re = regex::Regex::new(r"pub\s+ip:\s+String").unwrap();
            assert!(re.is_match(&contents));

            let re = regex::Regex::new(r"pub\s+port:\s+i64").unwrap();
            assert!(re.is_match(&contents));

            // check for renamed fields
            assert!(contents.contains("#[serde(rename = \"ip\")]"));
            assert!(contents.contains("#[serde(rename = \"anon_key2\")]"));

            // check for enum variants
            assert!(contents.contains("Dev,"));
            assert!(contents.contains("Production,"));

            // collecting all fields in Example struct to verify all are present
            let expected_example_fields = HashSet::from([
                "ip", "port", "ints", "birds", "mapping", "anon_map", "database", "mode",
            ]);

            let example_struct_re =
                regex::Regex::new(r"pub struct Example \{([\s\S]*?)\}").unwrap();

            let field_re = regex::Regex::new(r"pub\s+(\w+):").unwrap();

            if let Some(captures) = example_struct_re.captures(&contents) {
                let struct_body = captures.get(1).unwrap().as_str();
                let found_fields: HashSet<&str> = field_re
                    .captures_iter(struct_body)
                    .map(|cap| cap.get(1).unwrap().as_str())
                    .collect();

                assert_eq!(found_fields, expected_example_fields);
            } else {
                panic!("Could not find Example struct in generated code");
            }
        }

        #[test]
        fn test_deserialize_generated_code() {
            mod expected {
                use crate as rpkl;

                /* Generated by rpkl */

                #[derive(Debug, serde::Deserialize, serde::Serialize)]
                pub struct Example {
                    #[serde(rename = "ip")]
                    pub ip: String,
                    pub port: i64,
                    pub ints: std::ops::Range<i64>,
                    pub birds: Vec<rpkl::Value>,
                    pub mapping: rpkl::Value,
                    pub anon_map: example::AnonMap,
                    pub database: example::Database,
                    pub mode: example::Mode,
                }

                pub mod example {
                    #[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
                    pub struct AnonMap {
                        pub anon_key: String,
                        #[serde(rename = "anon_key2")]
                        pub anon_key_2: String,
                    }

                    #[derive(Debug, serde::Deserialize, serde::Serialize)]
                    pub struct Database {
                        pub username: String,
                        pub password: String,
                    }

                    #[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
                    pub enum Mode {
                        #[default]
                        Dev,
                        Production,
                    }
                }
            }

            let _ = crate::from_config::<expected::Example>(pkl_tests_file("example.pkl")).unwrap();
        }
    }
}

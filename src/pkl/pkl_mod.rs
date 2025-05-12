use super::internal::ObjectMember;

#[derive(Debug)]
pub struct PklMod {
    pub(crate) _module_name: String,
    pub(crate) _module_uri: String,
    pub(crate) members: Vec<ObjectMember>,
}

#[cfg(feature = "codegen")]
pub mod codegen {
    use convert_case::{Case, Casing};
    use std::{collections::HashSet, path::PathBuf};

    use std::io::Write;

    use crate::pkl::internal::{IPklValue, ObjectMember, PklNonPrimitive, PklPrimitive};

    use super::PklMod;
    use crate::Result;

    /// Config to modify the code generated from [`PklMod::codegen`].
    #[derive(Default)]
    pub struct CodegenOptions {
        type_attributes: Vec<(String, String)>,
        field_attributes: Vec<(String, String)>,
        enums: Vec<(String, String)>,
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
    }

    impl PklMod {
        /// By default, all structs are generated with `Debug`, `serde::Deserialize` and `serde::Serialize` attributes.
        ///
        /// To modify the generated code,
        /// use [`CodegenOptions`] to add additional attributes to the generated structs and fields.
        pub fn codegen(&self, options: Option<CodegenOptions>) -> Result<()> {
            let options = options.unwrap_or_default();
            let path = PathBuf::from("./generated");
            std::fs::create_dir_all(&path)?;
            let file = std::fs::File::create(path.join("mod.rs"))?;

            let module_name = &self._module_name;

            let mut writer = std::io::BufWriter::new(file);

            writeln!(writer, "/* Generated by rpkl */\n")?;

            let mut generated_structs = HashSet::new();

            let (code, deps) = PklMod::generate_struct(
                module_name,
                &self.members,
                false,
                module_name,
                true,
                &mut generated_structs,
                &options,
            )?;

            writeln!(writer, "{code}")?;

            if !deps.is_empty() {
                writeln!(
                    writer,
                    "pub mod {} {{",
                    self._module_name.to_case(Case::Snake)
                )?;
                // TODO: reimplement
                // writeln!(writer, "\tuse super::*;\n")?;
                for dep in deps.iter() {
                    dep.lines().for_each(|line| {
                        writeln!(writer, "\t{line}").unwrap();
                    });
                }

                writeln!(writer, "}}")?;
            }

            Ok(())
        }

        // TODO: refactor
        fn field_type_from_pkl_value(value: &IPklValue) -> String {
            match value {
                IPklValue::NonPrimitive(PklNonPrimitive::List(_, _)) => {
                    // todo!("print list types");
                    "Vec<rpkl::Value>".to_string()
                }
                IPklValue::NonPrimitive(PklNonPrimitive::Set(_, _)) => {
                    // todo!("print set types");
                    // warn!
                    "Vec<rpkl::Value>".to_string()
                }
                IPklValue::NonPrimitive(PklNonPrimitive::Mapping(_, _m)) => {
                    "rpkl::Value".to_string()
                }
                IPklValue::Primitive(x) => (match x {
                    PklPrimitive::Int(_) => "i64",
                    PklPrimitive::Float(_) => "f64",
                    PklPrimitive::String(_) => "String",
                    PklPrimitive::Boolean(_) => "bool",
                    PklPrimitive::Null => "Option<rpkl::Value>",
                })
                .to_string(),
                IPklValue::NonPrimitive(PklNonPrimitive::IntSeq(_, _, _)) => {
                    "std::ops::Range<i64>".to_string()
                }
                t => unimplemented!("implement codegen for other non-primitive types, {:?}", t),
            }
        }

        fn generate_field(
            (member_ident, member_value): (&str, &IPklValue),
            snake_case_field_name: &str,
            top_level_module_name: &str,
            deps: &mut Vec<String>,
            generated_structs: &mut HashSet<String>,
            options: &CodegenOptions,
            parent_struct_ident: &str,
        ) -> Result<String> {
            let mut field = String::new();

            if let IPklValue::NonPrimitive(PklNonPrimitive::TypedDynamic(
                _,
                _,
                _,
                dynamic_members,
            )) = member_value
            {
                let (dep, child_deps) = PklMod::generate_struct(
                    member_ident,
                    dynamic_members,
                    true,
                    top_level_module_name,
                    false,
                    generated_structs,
                    options,
                )?;
                deps.push(dep);
                deps.extend(child_deps);

                // add the field
                let upper_camel = member_ident.to_case(Case::UpperCamel);
                let rename = match member_ident == upper_camel {
                    true => Some(format!("#[serde(rename = \"{upper_camel}\")]\n",)),
                    false => None,
                };

                if let Some(rename) = &rename {
                    field.push_str(&format!("\t{rename}"));
                }

                field.push_str(&format!(
                    "\tpub {snake_case_field_name}: {}::{upper_camel},\n",
                    top_level_module_name.to_case(Case::Snake),
                ));

                return Ok(field);
            }

            let field_modifier = format!("{parent_struct_ident}.{snake_case_field_name}");

            if let Some(attr) = options.find_enum(&field_modifier) {
                let __enum = Self::generate_enum(
                    member_ident,
                    attr.split(",").map(|s| s.trim().to_string()).collect(),
                    true,
                    generated_structs,
                    options,
                )?;
                deps.push(__enum);
                field.push_str(&format!(
                    "\tpub {snake_case_field_name}: {}::{},\n",
                    top_level_module_name.to_case(Case::Snake),
                    member_ident.to_case(Case::UpperCamel),
                ));
                return Ok(field);
            }

            if let Some(attr) = options.find_field_attribute(&field_modifier) {
                field.push_str(&format!("\t{attr}\n"));
            }

            let rename = match snake_case_field_name == member_ident {
                true => None,
                false => Some(format!("#[serde(rename = \"{member_ident}\")]\n")),
            };

            if let Some(rename) = &rename {
                field.push_str(&format!("\t{rename}"));
            }

            field.push_str(&format!(
                "\tpub {snake_case_field_name}: {},\n",
                &PklMod::field_type_from_pkl_value(member_value)
            ));

            Ok(field)
        }

        fn generate_enum(
            enum_ident: &str,
            variants: Vec<String>,
            is_dependency: bool,
            // top_level_module_name: &str,
            generated_structs: &mut HashSet<String>,
            // field_modifier: &str,
            options: &CodegenOptions,
        ) -> Result<String> {
            let upper_camel = enum_ident.to_case(Case::UpperCamel);
            if generated_structs.contains(&upper_camel) {
                return Ok(String::new());
            }
            generated_structs.insert(upper_camel.clone());

            let mut code = String::new();

            if let Some(attr) = options.find_type_attribute(&upper_camel) {
                code.push_str(&format!("{attr}\n"));
            }

            code.push_str("#[derive(Debug, serde::Deserialize, serde::Serialize)]\n");

            if is_dependency {
                // code.push_str(&format!("pub(crate) struct {upper_camel} {{\n")); // TODO: revisit this
                code.push_str(&format!("pub enum {upper_camel} {{\n"));
            } else {
                code.push_str(&format!("pub enum {upper_camel} {{\n"));
            }

            for variant in variants.iter() {
                let variant_ident = variant.to_case(Case::UpperCamel);
                let varient_modifier_key = format!("{upper_camel}.{variant_ident}");
                if let Some(attrs) = options
                    .field_attributes
                    .iter()
                    .find(|(name, _)| *name == varient_modifier_key)
                {
                    code.push_str(&format!("\t{}\n", attrs.1));
                }
                code.push_str(&format!("\t{variant_ident},\n"));
            }

            code.push_str("}\n\n");

            Ok(code)
        }

        fn generate_struct(
            struct_ident: &str,
            members: &[ObjectMember],
            is_dependency: bool,
            top_level_module_name: &str,
            pub_struct: bool,
            generated_structs: &mut HashSet<String>,
            options: &CodegenOptions,
        ) -> Result<(String, Vec<String>)> {
            let upper_camel = struct_ident.to_case(Case::UpperCamel);
            if generated_structs.contains(&upper_camel) {
                return Ok((String::new(), vec![]));
            }
            generated_structs.insert(upper_camel.clone());

            let mut code = String::new();

            if let Some(attr) = options.find_type_attribute(&upper_camel) {
                code.push_str(&format!("{attr}\n"));
            }
            code.push_str("#[derive(Debug, serde::Deserialize, serde::Serialize)]\n");

            if is_dependency {
                // code.push_str(&format!("pub(crate) struct {upper_camel} {{\n")); // TODO: revisit this
                code.push_str(&format!("pub struct {upper_camel} {{\n"));
            } else if pub_struct {
                code.push_str(&format!("pub struct {upper_camel} {{\n"));
            } else {
                code.push_str(&format!("struct {upper_camel} {{\n"));
            }

            // code.push_str(&format!("struct {} {{\n", struct_ident));

            let mut deps = vec![];
            for member in members.iter() {
                let member_ident = member.get_ident();
                let member_value = member.get_value();
                let snake_case_field_name = member_ident.to_case(Case::Snake);
                let field = PklMod::generate_field(
                    (member_ident, member_value),
                    &snake_case_field_name,
                    top_level_module_name,
                    &mut deps,
                    generated_structs,
                    options,
                    &upper_camel,
                )?;
                code.push_str(&field);
            }

            code.push_str("}\n\n");

            Ok((code, deps))
        }
    }
}

use std::collections::HashSet;

use convert_case::{Case, Casing};

use crate::pkl::PklPrimitive;

use super::{IPklValue, ObjectMember, PklNonPrimitive};
use std::io::Write;

pub struct PklMod {
    pub _module_name: String,
    pub _module_uri: String,
    // pub _members: Vec<PklNonPrimitive>,
    // pub _mappings: HashMap<String, IPklValue>,
    pub(crate) members: Vec<ObjectMember>,
}

// TODO: refactor
/// Codegen
impl PklMod {
    pub fn codegen(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all("./src/gen")?;
        let file = std::fs::File::create("./src/gen/mod.rs")?;

        let module_name = &self._module_name;

        let mut writer = std::io::BufWriter::new(file);

        let mut generated_structs = HashSet::new();

        let (code, deps) = PklMod::generate_struct(
            &module_name,
            self.members.clone(),
            false,
            &module_name,
            true,
            &mut generated_structs,
        )?;

        writeln!(writer, "{}", code,)?;

        if !deps.is_empty() {
            writeln!(writer, "mod {} {{", self._module_name.to_case(Case::Snake))?;
            writeln!(writer, "\tuse super::*;\n")?;
            for dep in deps.iter() {
                dep.lines().for_each(|line| {
                    writeln!(writer, "\t{}", line).unwrap();
                });
            }

            writeln!(writer, "}}")?;
        }

        Ok(())
    }

    fn field_type_from_pkl_value(value: &IPklValue) -> String {
        match value {
            IPklValue::NonPrimitive(PklNonPrimitive::List(_, _)) => {
                // todo!("print list types");
                format!("Vec<{}>", "serde_json::Value")
            }
            IPklValue::NonPrimitive(PklNonPrimitive::Set(_, _)) => {
                // todo!("print set types");
                // warn!
                format!("Vec<{}>", "serde_json::Value")
            }
            IPklValue::NonPrimitive(PklNonPrimitive::Mapping(_, _m)) => {
                // todo!("print mapping types");
                format!("{}", "serde_json::Value")
            }
            IPklValue::Primitive(x) => {
                format!(
                    "{}",
                    match x {
                        PklPrimitive::Int(_) => "i64",
                        PklPrimitive::Float(_) => "f64",
                        PklPrimitive::String(_) => "String",
                        PklPrimitive::Bool(_) => "bool",
                        PklPrimitive::Null => "Option<serde_json::Value>",
                    }
                )
            }
            t => todo!("implement codegen for other non-primitive types, {:?}", t),
        }
    }

    fn generate_field(
        member_ident: &str,
        member_value: &IPklValue,
        snake_case_field_name: &str,
        top_level_module_name: &str,
        deps: &mut Vec<String>,
        generated_structs: &mut HashSet<String>,
    ) -> anyhow::Result<String> {
        let mut field = String::new();

        if let IPklValue::NonPrimitive(PklNonPrimitive::TypedDynamic(_, _, _, d)) = member_value {
            let (dep, child_deps) = PklMod::generate_struct(
                &member_ident,
                d.clone(),
                true,
                top_level_module_name,
                false,
                generated_structs,
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

        let rename = match snake_case_field_name == member_ident {
            true => None,
            false => Some(format!("#[serde(rename = \"{member_ident}\")]\n")),
        };

        if let Some(rename) = &rename {
            field.push_str(&format!("\t{}", rename));
        }

        field.push_str(&format!(
            "\tpub {snake_case_field_name}: {},\n",
            &PklMod::field_type_from_pkl_value(member_value)
        ));

        Ok(field)
    }

    fn generate_struct(
        struct_ident: &str,
        members: Vec<ObjectMember>,
        is_dependency: bool,
        top_level_module_name: &str,
        pub_struct: bool,
        generated_structs: &mut HashSet<String>,
    ) -> anyhow::Result<(String, Vec<String>)> {
        let upper_camel = struct_ident.to_case(Case::UpperCamel);
        if generated_structs.contains(&upper_camel) {
            return Ok((String::new(), vec![]));
        }
        generated_structs.insert(upper_camel.clone());

        let mut code = String::new();

        code.push_str(&format!(
            "#[derive(serde::Deserialize, serde::Serialize)]\n"
        ));

        if is_dependency {
            code.push_str(&format!("pub(crate) struct {upper_camel} {{\n"));
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
                &member_ident,
                &member_value,
                &snake_case_field_name,
                top_level_module_name,
                &mut deps,
                generated_structs,
            )?;
            code.push_str(&field);
        }

        code.push_str("}\n\n");

        Ok((code, deps))
    }
}

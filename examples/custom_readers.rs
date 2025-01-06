use std::path::PathBuf;

use rpkl::{
    api::external_reader::{
        outgoing::PathElements, reader::ExternalReaderRuntime, PklModuleReader, PklResourceReader,
    },
    EvaluatorOptions,
};
use serde::Deserialize;

pub struct LdapReader;
pub struct LdapsReader;

pub struct ModuleReader;

impl PklResourceReader for LdapReader {
    fn scheme(&self) -> &str {
        "ldap"
    }

    fn read(&self, uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(uri.bytes().collect())
    }

    fn list(&self, _uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

impl PklResourceReader for LdapsReader {
    fn scheme(&self) -> &str {
        "ldaps"
    }

    fn read(&self, uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(uri.bytes().collect())
    }

    fn list(&self, uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

impl PklModuleReader for ModuleReader {
    fn scheme(&self) -> &str {
        "remote"
    }

    fn read(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("".to_string())
    }

    fn is_local(&self) -> bool {
        true
    }

    fn list(&self, _uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

pub fn main() {
    #[derive(Debug, Deserialize)]
    struct Config {
        username: String,
        ldap_email: String,
        ldaps_email: String,
    }

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("pkl")
        .join("external-reader.pkl");

    let options = EvaluatorOptions::default()
        .properties([("name", "Ferris")])
        .add_client_module_readers(ModuleReader)
        .add_client_resource_readers((LdapReader, LdapsReader));
    let config: Config = rpkl::from_config_with_options(path, options).unwrap();

    println!("{:?}", config);
}

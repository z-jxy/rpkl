//! Example of creating an external reader that can be used with `pkl` directly.

use rpkl::api::external_reader::*;

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

    fn list(&self, _uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

impl PklModuleReader for ModuleReader {
    fn scheme(&self) -> &str {
        "remote"
    }

    fn read(&self, _uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("".to_string())
    }

    fn is_local(&self) -> bool {
        true
    }

    fn list(&self, _uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

/// To test this, compile the example `cargo build --example external_resource_reader`
/// and run the following command:
/// ```bash
/// pkl eval tests/pkl/external-reader.pkl \
/// --external-resource-reader ldap=target/debug/examples/external_resource_reader \
/// --external-module-reader remote=target/debug/examples/external_resource_reader \
/// --external-resource-reader ldaps=target/debug/examples/external_resource_reader
/// ```
pub fn main() {
    _ = ExternalReaderRuntime::new()
        .add_resource_readers((LdapReader, LdapsReader))
        .add_module_readers(ModuleReader)
        .run();
}

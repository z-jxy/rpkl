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

pub fn main() {
    #[cfg(feature = "trace")]
    {
        use tracing_subscriber::{
            fmt, fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
        };
        // Create a log file
        let log_file = std::fs::File::create("output.log").unwrap();

        // Set up a tracing subscriber
        let file_layer = fmt::layer()
            .with_writer(log_file) // Write logs to the file
            .with_ansi(false); // Disable ANSI colors for file logs

        // Use environment variables to set log levels, or default to `info`
        // let filter_layer =
        //     EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(file_layer)
            // .with(filter_layer)
            .init();
    }

    _ = ExternalReaderRuntime::new()
        .add_resource_readers((LdapReader, LdapsReader))
        .add_module_readers(ModuleReader)
        .run();
}

pub mod incoming;
pub mod outgoing;
pub mod reader;

// pub use incoming::ReadResource;

pub enum ReaderType {
    Module,
    Resource,
}

pub trait ExternalReaderClient {
    // const READER_TYPE: ReaderType;

    fn reader_type(&self) -> ReaderType;

    // fn run();
    // fn read(uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn read(&self, uri: &str) -> Option<Vec<u8>>;
}

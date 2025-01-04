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

    /// Scheme returns the scheme part of the URL that this reader can read.
    /// The value should be the URI scheme up to (not including) `:`
    fn scheme(&self) -> &str;

    /// Schemes returns a list of schemes that this reader can read. Used when initalizing the reader. First the result from `scheme` is checked, otherwise this list is checked.
    // fn schemes(&self) -> Vec<&str> {
    //     vec![self.scheme()]
    // }

    fn has_hierarchical_uris(&self) -> bool {
        false
    }

    fn is_globbable(&self) -> bool {
        false
    }

    fn reader_type(&self) -> ReaderType;

    // fn read(uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    // fn read(&self, uri: &str) -> Option<Vec<u8>>;
    fn read(&self, uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    // fn read<'a>(&'a self, uri: &str) -> Option<&'a [u8]>;
}

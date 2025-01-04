pub mod incoming;
pub mod outgoing;
pub mod reader;

// pub use incoming::ReadResource;

pub enum ReaderType {
    Module,
    Resource,
}

pub trait PklReader {
    // const READER_TYPE: ReaderType;

    /// Scheme returns the scheme part of the URL that this reader can read.
    /// The value should be the URI scheme up to (not including) `:`
    fn scheme(&self) -> &str;

    /// Tells whether the path part of ths URI has a
    /// [hier-part](https://datatracker.ietf.org/doc/html/rfc3986#section-3).
    ///
    /// An example of a hierarchical URI is `file:///path/to/my/file`, where
    /// `/path/to/my/file` designates a nested path through the `/` character.
    ///
    /// An example of a non-hierarchical URI is `pkl.base`, where the `base` does not denote
    /// any form of hierarchy.
    fn has_hierarchical_uris(&self) -> bool {
        false
    }

    /// Tells whether this reader supports globbing.
    fn is_globbable(&self) -> bool {
        false
    }

    fn reader_type(&self) -> ReaderType;

    // fn read(uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    // fn read(&self, uri: &str) -> Option<Vec<u8>>;
    fn read(&self, uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    // fn read<'a>(&'a self, uri: &str) -> Option<&'a [u8]>;
}

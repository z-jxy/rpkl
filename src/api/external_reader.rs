use outgoing::PathElements;

pub mod incoming;
pub mod outgoing;
pub mod reader;

// pub use incoming::ReadResource;

pub trait PklReader {
    fn read(&self);
    fn list(&self);
}

pub trait PklResourceReader {
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

    fn read(&self, uri: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn list(&self, uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>>;
}

pub trait PklModuleReader {
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

    fn read(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>>;
    fn list(&self, uri: &str) -> Result<Vec<PathElements>, Box<dyn std::error::Error>>;

    fn is_local(&self) -> bool;
}

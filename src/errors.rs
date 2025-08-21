//! Error and result type derived using the [`thiserror`] crate.
//!
//! [`thiserror`]: https://docs.rs/thiserror/

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("invalid scheme: {0:?}")]
    InvalidScheme(String),
    #[error("invalid type: {0:?}")]
    InvalidType(String),
    #[error("invalid key: {0:?}")]
    InvalidKey(String),
    #[error("missing name")]
    MissingName,
    #[error("no namespace allowed for type {0:?}")]
    TypeProhibitsNamespace(String),
    #[error("invalid namespace component: {0:?}")]
    InvalidNamespaceComponent(String),
    #[error("missing scheme")]
    MissingScheme,
    #[error("missing type")]
    MissingType,
    #[error("invalid subpath segment: {0:?}")]
    InvalidSubpathSegment(String),
    #[error("utf-8 decoding failed")]
    DecodingError(#[source] std::str::Utf8Error),
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::DecodingError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

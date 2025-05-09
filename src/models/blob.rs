//! Blob represents a binary large object (BLOB) in a Git-like system.

use std::{borrow::Cow, fmt::Display};

use super::object::Sha1Able;
use bincode::{Decode, Encode};
use sha1::Digest;

#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub enum Blob {
    /// Binary data
    ///
    /// Some of files may not be a valid UTF-8 file, and we won't want to count
    /// line differences of them
    ///
    /// Not all byte slices are valid [String]s, however: strings are required
    /// to be valid UTF-8. If we store them in a [String], from_utf8_lossy()
    /// will replace any invalid UTF-8 sequences with `U+FFFD REPLACEMENT
    /// CHARACTER`, which looks like this: �
    Binary(Vec<u8>),
    /// Text data
    ///
    /// To represent a valid UTF-8 file
    Text(String),
}

impl From<String> for Blob {
    fn from(value: String) -> Self {
        Blob::Text(value)
    }
}

impl From<&str> for Blob {
    fn from(value: &str) -> Self {
        Blob::Text(value.to_string())
    }
}

impl From<Vec<u8>> for Blob {
    fn from(value: Vec<u8>) -> Self {
        Blob::Binary(value)
    }
}

impl From<&[u8]> for Blob {
    fn from(value: &[u8]) -> Self {
        Blob::Binary(value.into())
    }
}

impl Blob {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Blob::Binary(data) => data,
            Blob::Text(text) => text.as_bytes(),
        }
    }

    /// Converts a slice of bytes to a string, including invalid characters. Not
    /// all [Blob]s are valid strings, however: strings are required to be valid
    /// UTF-8. During this conversion, from_utf8_lossy() will replace any
    /// invalid UTF-8 sequences with `U+FFFD REPLACEMENT CHARACTER`, which looks
    /// like this: �
    ///
    /// So you may need check if the blob is a [Blob::Binary] first
    pub fn as_string(&self) -> Cow<str> {
        match self {
            Blob::Binary(data) => String::from_utf8_lossy(data),
            Blob::Text(text) => text.into(),
        }
    }
}

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl Sha1Able for Blob {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(self.as_bytes());
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_sha1() {
        let blob = Blob::from("hello world");
        assert_eq!(blob.sha1(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
        let blob = Blob::from("hello world!");
        assert_eq!(blob.sha1(), "430ce34d020724ed75a196dfc2ad67c77772d169");
        let blob = Blob::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(blob.sha1(), "11966ab9c099f8fabefac54c08d5be2bd8c903af");
    }
}

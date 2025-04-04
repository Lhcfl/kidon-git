//! Blob represents a binary large object (BLOB) in a Git-like system.

use super::object::Sha1Able;
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob(String);

impl From<String> for Blob {
    fn from(value: String) -> Self {
        Blob(value)
    }
}
impl From<&str> for Blob {
    fn from(value: &str) -> Self {
        Blob(value.to_string())
    }
}

impl Sha1Able for Blob {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(self.0.as_bytes());
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_sha1() {
        let mut blob = Blob::from("hello world");
        assert_eq!(blob.sha1(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
        blob.0.push('!');
        assert_eq!(blob.sha1(), "430ce34d020724ed75a196dfc2ad67c77772d169");
    }
}

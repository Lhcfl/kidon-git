use serde::{Deserialize, Serialize};
use sha1::Digest;

use crate::traits::Sha1Able;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob(String);

impl Sha1Able for Blob {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(self.0.as_bytes());
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

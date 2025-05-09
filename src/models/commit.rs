//! Commit object

use std::fmt::Display;

use super::object::{ObjectSha1, Sha1Able};

use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Commit {
    /// Commit tree
    pub tree: ObjectSha1,
    /// Privous commit
    pub parent: Option<ObjectSha1>,
    /// Commit time
    pub timestamp: (i64, u32),
    /// Commit message.  
    /// The first line is the summary, and the rest is the body.
    pub message: String,
}

pub struct CommitBuilder {
    pub tree: ObjectSha1,
    pub parent: Option<ObjectSha1>,
    pub message: String,
}

impl Commit {
    pub fn time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.timestamp.0, self.timestamp.1).expect("Invalid timestamp")
    }

    pub fn new(by: CommitBuilder) -> Commit {
        let now = Utc::now();
        Commit {
            tree: by.tree,
            parent: by.parent,
            timestamp: (now.timestamp(), now.timestamp_subsec_nanos()),
            message: by.message,
        }
    }
}

impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                write!(f, "{}", json)
            }
            Err(e) => {
                write!(f, "unexpected: failed to serialize the commit: {}", e)
            }
        }
    }
}

impl Sha1Able for Commit {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(self.tree.as_bytes());
        hasher.update(
            self.parent
                .as_ref()
                .map(|s| s.as_bytes())
                .unwrap_or("".as_bytes()),
        );
        hasher.update(self.timestamp.0.to_le_bytes());
        hasher.update(self.timestamp.1.to_le_bytes());
        hasher.update(self.message.as_bytes());
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_sha1() {
        let mut commit = Commit {
            tree: "tree_hash".into(),
            parent: Some("parent_hash".into()),
            timestamp: (0, 0),
            message: "commit message".into(),
        };

        let sha1 = commit.sha1();
        assert_eq!(sha1, "9fdae82bc2f37cc414b82bb7255c48461ee8c096");
        commit.tree = "new_tree_hash".into();
        assert_ne!(sha1, commit.sha1());
    }
}

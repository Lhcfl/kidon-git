//! Commit object

use super::object::{ObjectSha1, Sha1Able};
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    /// Commit tree
    pub tree: ObjectSha1,
    /// Privous commit
    pub parent: Option<ObjectSha1>,
    /// Commit time
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Commit message.  
    /// The first line is the summary, and the rest is the body.
    pub message: String,
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
        hasher.update(self.timestamp.timestamp().to_le_bytes());
        hasher.update(self.message.as_bytes());
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_commit_sha1() {
        let mut commit = Commit {
            tree: "tree_hash".into(),
            parent: Some("parent_hash".into()),
            timestamp: SystemTime::UNIX_EPOCH.into(),
            message: "commit message".into(),
        };

        let sha1 = commit.sha1();
        assert_eq!(sha1, "1f665dd06ee620c7d553a05d9c8fea37495567bd");
        commit.tree = "new_tree_hash".into();
        assert_ne!(sha1, commit.sha1());
    }
}

//! Commit object

use super::object::{ObjectSha1, Sha1Able};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub tree: ObjectSha1,
    pub parent: Option<ObjectSha1>,
    pub timestamp: SystemTime,
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
        hasher.update(
            self.timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("commit has a invalid timestamp")
                .as_millis()
                .to_le_bytes(),
        );
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
            timestamp: SystemTime::UNIX_EPOCH,
            message: "commit message".into(),
        };

        let sha1 = commit.sha1();
        assert_eq!(sha1, "97225a7022fe0c8774c228cc13bbe9d0363342b1");
        commit.tree = "new_tree_hash".into();
        assert_ne!(sha1, commit.sha1());
    }
}

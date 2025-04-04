//! git objects

use super::{blob::Blob, commit::Commit, tree::Tree};
use crate::traits::{DirContainer, Store};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

#[enum_dispatch]
pub trait Sha1Able {
    fn sha1(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSha1(String);

impl From<String> for ObjectSha1 {
    fn from(value: String) -> Self {
        ObjectSha1(value)
    }
}

impl From<&str> for ObjectSha1 {
    fn from(value: &str) -> Self {
        ObjectSha1(value.to_string())
    }
}

impl Deref for ObjectSha1 {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[enum_dispatch(Sha1Able)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl DirContainer for Object {
    const DIRECTORY: &'static str = "objects";
}

impl Store for Object {
    fn loaction(&self) -> PathBuf {
        let sha1 = self.sha1();
        let dir = sha1.chars().take(2).collect::<String>();
        Path::new(Self::DIRECTORY).join(dir).join(&sha1[2..])
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::models::{object::Sha1Able, tree::TreeLine, tree::TreeLineKind};

    use super::{Blob, Commit, Object, Tree};

    #[test]
    fn object_sha1_should_eq_inner_sha1() {
        let blob = Blob::from("hello world");
        assert_eq!(blob.sha1(), Object::from(blob).sha1());

        let commit = Commit {
            tree: "tree_hash".into(),
            parent: Some("parent_hash".into()),
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            message: "commit message".to_string(),
        };
        assert_eq!(commit.sha1(), Object::from(commit).sha1());

        let tree = Tree {
            objects: vec![TreeLine {
                kind: TreeLineKind::File,
                name: "file.txt".to_string(),
                sha1: "file_sha1".into(),
            }],
        };
        assert_eq!(tree.sha1(), Object::from(tree).sha1());
    }

    #[test]
    fn object_serialize() {
        let blob = Blob::from("hello world");
        let object = Object::from(blob.clone());
        let serialized = serde_json::to_string(&object).unwrap();
        assert_eq!(
            serialized,
            json!({
                "Blob": "hello world"
            })
            .to_string()
        );

        let commit = Commit {
            tree: "tree_hash".into(),
            parent: Some("parent_hash".into()),
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            message: "commit message".to_string(),
        };
        let object = Object::from(commit.clone());
        let serialized = serde_json::to_value(object).unwrap();
        assert_eq!(
            serialized,
            json!({
                "Commit": {
                    "tree": "tree_hash",
                    "parent": "parent_hash",
                    "timestamp": {
                        "secs_since_epoch": 0,
                        "nanos_since_epoch": 0
                    },
                    "message": "commit message",
                }
            })
        );

        let tree = Tree {
            objects: vec![TreeLine {
                kind: TreeLineKind::File,
                name: "file.txt".to_string(),
                sha1: "file_sha1".into(),
            }],
        };
        let object = Object::from(tree.clone());
        let serialized = serde_json::to_value(object).unwrap();
        assert_eq!(
            serialized,
            json!({
                "Tree": {
                    "objects": [
                        {
                            "kind": "File",
                            "name": "file.txt",
                            "sha1": "file_sha1"
                        }
                    ]
                }
            })
        );
    }
}

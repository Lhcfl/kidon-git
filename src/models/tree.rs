//! A Tree stores the structure of a directory in a git repository.
//! It contains a list of TreeLine objects, each representing a blob or another tree.

use super::object::{ObjectSha1, Sha1Able};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeLineKind {
    File,
    Executable,
    Symlink,
    Tree,
}

impl Display for TreeLineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeLineKind::File => write!(f, "100644 blob"),
            TreeLineKind::Executable => write!(f, "100755 blob"),
            TreeLineKind::Symlink => write!(f, "120000 blob"),
            TreeLineKind::Tree => write!(f, "040000 tree"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeLine {
    pub kind: TreeLineKind,
    pub name: String,
    pub sha1: ObjectSha1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub objects: Vec<TreeLine>,
}

impl Sha1Able for Tree {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        for line in &self.objects {
            hasher.update(line.kind.to_string().as_bytes());
            hasher.update(line.name.as_bytes());
            hasher.update(line.sha1.as_bytes());
        }
        base16ct::lower::encode_string(&hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sha1() {
        let mut tree = Tree {
            objects: vec![
                TreeLine {
                    kind: TreeLineKind::File,
                    name: "file.txt".to_string(),
                    sha1: "abc123".into(),
                },
                TreeLine {
                    kind: TreeLineKind::File,
                    name: "dir".to_string(),
                    sha1: "def456".into(),
                },
            ],
        };

        let sha1 = tree.sha1();
        assert_eq!(sha1, "00bfe760502a870dff983987f29dcf6e8dd76495");
        tree.objects.push(TreeLine {
            kind: TreeLineKind::Executable,
            name: "new_file.exe".to_string(),
            sha1: "xyz789".into(),
        });

        assert_ne!(sha1, tree.sha1());
    }
}

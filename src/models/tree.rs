use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sha1::Digest;

use crate::traits::Sha1Able;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeLineKind {
    Blob,
    Tree,
}

impl Display for TreeLineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeLineKind::Blob => write!(f, "blob"),
            TreeLineKind::Tree => write!(f, "tree"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeLine {
    pub mode: String,
    pub kind: TreeLineKind,
    pub name: String,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub objects: Vec<TreeLine>,
}

impl Sha1Able for Tree {
    fn sha1(&self) -> String {
        let mut hasher = sha1::Sha1::new();
        for line in &self.objects {
            hasher.update(line.mode.as_bytes());
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
                    mode: "100644".to_string(),
                    kind: TreeLineKind::Blob,
                    name: "file.txt".to_string(),
                    sha1: "abc123".to_string(),
                },
                TreeLine {
                    mode: "40000".to_string(),
                    kind: TreeLineKind::Tree,
                    name: "dir".to_string(),
                    sha1: "def456".to_string(),
                },
            ],
        };

        let sha1 = tree.sha1();
        assert_eq!(sha1, "d413f84eb641f0401172f16cf91c3d0d7ffff90a");
        tree.objects.push(TreeLine {
            mode: "100644".to_string(),
            kind: TreeLineKind::Blob,
            name: "new_file.txt".to_string(),
            sha1: "xyz789".to_string(),
        });
        assert_ne!(sha1, tree.sha1());
    }
}

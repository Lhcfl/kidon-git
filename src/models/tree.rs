//! A Tree stores the structure of a directory in a git repository. It contains
//! a list of TreeLine objects, each representing a blob or another tree.

use super::object::{ObjectSha1, Sha1Able};
use bincode::{Decode, Encode};
use sha1::Digest;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, Hash)]
pub struct TreeLine {
    pub kind: TreeLineKind,
    pub name: String,
    pub sha1: ObjectSha1,
}

impl Display for TreeLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}    {}", self.kind, self.sha1, self.name)
    }
}

/// A normal tree is like a folder in a file system. for example a tree can be
///
/// ```txt
/// 100644 blob 0c12d336241a22c6b954d6fafdc998b6d9fb6e17    .gitignore
/// 040000 tree 90627a7a454193542b63eabb58a5e4ca28757313    .vscode
/// 100644 blob 44e34a5a2ce18b5b2adf49e31830279372d72f59    Cargo.lock
/// 100644 blob e3c210d6b009a9f124374249285ed150afc810e5    Cargo.toml
/// 100644 blob fbc9698bf694bdf8236057cc7c3a44d81d17dab5    README.md
/// 040000 tree 0e7c7eacd07aae9bfe41f3a88a9dab8b6401a5c9    src
/// ```
///
/// and you can recursively get the tree of `.vscode` by accessing [ObjectSha1]
/// `90627a7a454193542b63eabb58a5e4ca28757313`
///
/// ```txt
/// 100644 blob 770d907549efbbdce816734005b3ebc5364b3208    settings.json
/// ```
///
/// then you can get the content of `settings.json` by accessing [ObjectSha1]
/// `770d907549efbbdce816734005b3ebc5364b3208` which is a
/// [Blob](super::blob::Blob)
#[derive(Debug, Clone, Decode, Encode)]
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

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.objects {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl From<HashMap<String, TreeLine>> for Tree {
    fn from(value: HashMap<String, TreeLine>) -> Self {
        let mut ret = Tree {
            objects: value.into_values().collect(),
        };
        ret.objects.sort_by(|a, b| a.name.cmp(&b.name));
        ret
    }
}

impl Tree {
    pub fn into_map(self) -> HashMap<String, TreeLine> {
        self.objects
            .into_iter()
            .map(|line| (line.name.clone(), line))
            .collect()
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

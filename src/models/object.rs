//! git objects

use super::{blob::Blob, commit::Commit, tree::Tree};
use crate::traits::{Accessable, DirContainer, Store};
use bincode::{Decode, Encode};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    mem::transmute,
    ops::Deref,
    path::{Path, PathBuf},
};

#[enum_dispatch]
pub trait Sha1Able {
    /// sha1 of the sha1able object
    fn sha1(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode, Serialize, Deserialize, Hash)]
#[repr(transparent)]
pub struct ObjectSha1(String);

impl ObjectSha1 {
    fn splited(&self) -> (&str, &str) {
        (&self.0[0..2], &self.0[2..])
    }
}

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

impl From<&String> for &ObjectSha1 {
    fn from(value: &String) -> Self {
        // SAFETY: This is safe because ObjectSha1 is repr(transparent) over
        // String so the memory layout is the same.
        unsafe { transmute(value) }
    }
}

impl Deref for ObjectSha1 {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ObjectSha1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Decode, Encode)]
#[enum_dispatch(Sha1Able)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}

impl Object {
    pub fn object_type(&self) -> &'static str {
        match self {
            Object::Blob(_) => "blob",
            Object::Tree(_) => "tree",
            Object::Commit(_) => "commit",
        }
    }

    /// ### Panics
    /// If the object is not a blob, the function will panic.
    pub fn cast_blob(self) -> Blob {
        match self {
            Object::Blob(blob) => blob,
            _ => panic!("Object is not a Blob"),
        }
    }

    /// ### Panics
    /// If the object is not a tree, the function will panic.
    pub fn cast_tree(self) -> Tree {
        match self {
            Object::Tree(tree) => tree,
            _ => panic!("Object is not a Tree"),
        }
    }

    /// ### Panics
    /// If the object is not a commit, the function will panic.
    pub fn cast_commit(self) -> Commit {
        match self {
            Object::Commit(commit) => commit,
            _ => panic!("Object is not a Commit"),
        }
    }
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
    fn store(&self, root: &std::path::Path) -> std::io::Result<()> {
        let path = root.join(self.loaction());
        if let Some(parent) = path.parent() {
            // Safely ignores the error if the directory already exists
            let _ = std::fs::create_dir_all(parent);
        }
        let mut dst = fs::File::create(path)?;
        bincode::encode_into_std_write(self, &mut dst, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(())
    }
    fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let mut src = fs::File::open(path)?;
        let item = bincode::decode_from_std_read(&mut src, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(item)
    }
}

impl Accessable<ObjectSha1> for Object {
    fn path_of(by: &ObjectSha1) -> PathBuf {
        let (car, cdr) = by.splited();
        Path::new(Self::DIRECTORY).join(car).join(cdr)
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Blob(blob) => write!(f, "{}", blob),
            Object::Tree(tree) => write!(f, "{}", tree),
            Object::Commit(commit) => write!(f, "{}", commit),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::models::{object::Sha1Able, tree::TreeLine, tree::TreeLineKind};

    use super::{Blob, Commit, Object, Tree};

    #[test]
    fn object_sha1_should_eq_inner_sha1() {
        let blob = Blob::from("hello world");
        assert_eq!(blob.sha1(), Object::from(blob).sha1());

        let commit = Commit {
            tree: "tree_hash".into(),
            parent: Some("parent_hash".into()),
            timestamp: (0, 0),
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
}

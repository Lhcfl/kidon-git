//! stage files of the repo
//! The stage files are used to store the changes that are not yet committed.

use super::tree::{Tree, TreeLine};
use crate::traits::Store;
use bincode::{Decode, Encode};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Staging area is actually a special [Tree]  
/// Notice that the stage tree will never contains another [Tree],
/// which is the difference between a stage and a tree
/// For example, a stage can contains a line like this:
///
/// ```txt
/// 100644 blob 1234567890abcdef1234567890abcdef12345678    src/some_dir/file.txt
/// ```
///
/// but a tree can only contains a line like this:
///
/// ```txt
/// 040000 tree abcdef1234567890abcdef1234567890abcdef12    src
/// ``````
///
/// See [Tree] for details
#[derive(Encode, Decode)]
pub struct Stage(pub Tree);

impl Deref for Stage {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Store for Stage {
    fn loaction(&self) -> PathBuf {
        Path::new(Self::LOCATION).to_path_buf()
    }

    fn store(&self, root: &Path) -> io::Result<()> {
        let path = root.join(Self::LOCATION);
        if let Some(parent) = path.parent() {
            // Safely ignores the error if the directory already exists
            let _ = std::fs::create_dir_all(parent);
        }
        let mut dst = fs::File::create(path)?;
        bincode::encode_into_std_write(self, &mut dst, bincode::config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(())
    }

    fn load(path: &Path) -> io::Result<Self> {
        let mut src = fs::File::open(path)?;
        let item = bincode::decode_from_std_read(&mut src, bincode::config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(item)
    }
}

impl Stage {
    pub const LOCATION: &str = "index";

    pub fn empty() -> Self {
        Stage(Tree {
            objects: Vec::new(),
        })
    }
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

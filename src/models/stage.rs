//! stage files of the repo
//! The stage files are used to store the changes that are not yet committed.

use super::tree::Tree;
use crate::traits::Store;
use bincode::{Decode, Encode};
use std::{
    fs, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Staging area is actually a special [[Tree]]
#[derive(Encode, Decode)]
pub struct Stage(Tree);

impl Deref for Stage {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Stage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

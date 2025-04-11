//! stage files of the repo
//! The stage files are used to store the changes that are not yet committed.

use super::tree::Tree;
use crate::traits::Store;
use bincode::{Decode, Encode};
use std::{
    fs,
    ops::{Deref, DerefMut},
    path::Path,
};

/// Staging area is actually a special [[Tree]]
#[derive(Encode, Decode)]
pub struct Stage(Tree);

impl Stage {
    pub const LOCATION: &str = "index";

    pub fn empty() -> Self {
        Stage(Tree {
            objects: Vec::new(),
        })
    }
}

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
    fn loaction(&self) -> std::path::PathBuf {
        Path::new(Self::LOCATION).to_path_buf()
    }

    fn store(&self, root: &std::path::Path) -> std::io::Result<()> {
        let path = root.join(Self::LOCATION);
        if let Some(parent) = path.parent() {
            // Safely ignores the error if the directory already exists
            let _ = std::fs::create_dir_all(parent);
        }
        let mut dst = fs::File::open(path)?;
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

//! Staging area of the repo. The stage files are used to store the changes that
//! are not yet committed.

use super::tree::Tree;
use crate::models::Store;
use bincode::{Decode, Encode};
use std::{
    fmt::Display,
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

/// Staging area is actually a special [Tree]  
///
/// See [Tree] for details
///
/// # Mutablity
///
/// We don't allow directly [std::ops::DerefMut] for [Stage], because it's eazy
/// to insert a dumplicated line into the stage, which is not an expected
/// behavior. If you want to modify the stage, take the Tree by call
/// [crate::services::stage::StageService::into_muter]
#[derive(Encode, Decode)]
pub struct Stage(pub Tree);

impl Deref for Stage {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Store for Stage {
    fn location(&self) -> PathBuf {
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

//! A pointer to the currently active branch of the context (repository, remote, etc.)

use crate::traits::Store;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum HeadKind {
    Local,
    Remote(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Head {
    pub kind: HeadKind,
    pub branch: String,
}

impl Store for Head {
    fn loaction(&self) -> PathBuf {
        Path::new("HEAD").to_path_buf()
    }
}

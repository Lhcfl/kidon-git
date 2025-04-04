//! A pointer to the currently active branch of the context (repository, remote, etc.)

use crate::traits::Store;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::branch::Branch;

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

impl Head {
    fn get_branch(&self) -> Branch {
        match &self.kind {
            HeadKind::Local => Branch {
                name: self.branch.clone(),
            },
            HeadKind::Remote(remote) => Branch {
                name: format!("{}/{}", remote, self.branch),
            },
        }
    }
}

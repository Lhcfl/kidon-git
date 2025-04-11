//! A pointer to the currently active branch of the context (repository, remote,
//! etc.)

use crate::{
    serde_json_store,
    traits::{Accessable, Accessor, Store},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{branch::Branch, repo::WithRepoPath};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum HeadKind {
    Local,
    Remote(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Head {
    pub kind: HeadKind,
    pub branch_name: String,
}

impl Store for Head {
    fn loaction(&self) -> PathBuf {
        Path::new("HEAD").to_path_buf()
    }
    serde_json_store!();
}

impl<'r> WithRepoPath<'r, &Head> {
    /// Get the branch of the head
    pub fn branch(&self) -> WithRepoPath<'r, Accessor<String, Branch>> {
        self.wrap(Branch::accessor(&self.branch_name))
    }
}

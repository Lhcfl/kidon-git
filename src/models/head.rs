//! A pointer to the currently active branch of the context (repository, remote,
//! etc.)

use crate::{
    serde_json_store,
    traits::{Accessable, Accessor, Store},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{branch::Branch, repo::WithRepo};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum HeadKind {
    Local,
    Remote(String),
}

/// Head is a repo's `HEAD` file, pointers to a [Branch]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Head {
    pub kind: HeadKind,
    pub branch_name: String,
}

impl Store for Head {
    fn location(&self) -> PathBuf {
        Path::new("HEAD").to_path_buf()
    }
    serde_json_store!();
}

impl<'r> WithRepo<'r, &Head> {
    /// Get the branch of the head
    pub fn branch<'a>(&'a self) -> WithRepo<'r, Accessor<'a, String, Branch>> {
        self.wrap(Branch::accessor(&self.branch_name))
    }
}

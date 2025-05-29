//! A pointer to the currently active branch of the context (repository, remote,
//! etc.)

use crate::{
    serde_json_store,
    models::{Accessible, Store},
};
use serde::{Deserialize, Serialize};
use std::{io, path::{Path, PathBuf}};

use super::{branch::Branch, repo::WithRepo};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum HeadKind {
    Local,
    Remote(String),
}

/// Head is a repo's `HEAD` file, pointers to a [Branch]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    /// the branch may not exist, so we need to create it if it does not exist
    pub fn load_branch_or_create(&self) -> io::Result<(WithRepo<'r, Branch>, bool)> {
        let name= self.branch_name.as_str();
        let branch = self.wrap(Branch::accessor(&name)).load();
        match branch {
            Ok(branch) => Ok((branch, false)),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok((self.wrap(Branch::new(&self.branch_name)), true))
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn load_branch(&self) -> io::Result<WithRepo<'r, Branch>> {
        let name= self.branch_name.as_str();
        self.wrap(Branch::accessor(&name)).load()
    }
}

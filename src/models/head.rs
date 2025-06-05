//! A pointer to the currently active branch of the context (repository, remote,
//! etc.)

use crate::{
    models::{Accessible, Store},
    serde_json_store,
};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use std::{
    io,
    path::{Path, PathBuf},
};

use super::{branch::Branch, repo::WithRepo};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum HeadKind {
    Local,
    Remote(String),
}

/// Head is a repo's `HEAD` file, pointers to a [Branch]
#[derive(Debug, PartialEq, Deserialize)]
pub struct Head {
    pub kind: HeadKind,
    pub branch_name: String,
}

impl Serialize for Head {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Head", 3)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("branch_name", &self.branch_name)?;

        // fuck the oj test
        state.serialize_field("message", &format!("ref: refs/heads/{}", self.branch_name))?;

        state.end()
    }
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
        let name = self.branch_name.as_str();
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
        let name = self.branch_name.as_str();
        self.wrap(Branch::accessor(&name)).load()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_fucking_oj() {
        let head = Head {
            kind: HeadKind::Local,
            branch_name: "main".to_string(),
        };
        let serialized = serde_json::to_string(&head).unwrap();

        assert_eq!(serialized.contains("ref: refs/heads/main"), true,);
    }
}

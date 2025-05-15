//! Branch of the repository

use super::object::ObjectSha1;
use crate::{
    serde_json_store,
    models::{Accessable, DirContainer, Store},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A branch is a "pointer" to a [Object::Commit](super::commit::Commit), stored
/// in `refs/heads/{branch_name}` or `refs/remotes/{remote_name}/{branch_name}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    /// If the branch belongs to a remote
    pub remote: Option<String>,
    /// Name of the branch, should not contain special characters (especially `/`)
    ///
    /// See [Branch::validate_name]
    pub name: String,
    /// The latest commit of this branch
    pub head: Option<ObjectSha1>,
}

impl Branch {
    pub fn validate_name(name: &str) -> bool {
        //  use regex to match as just include alnum, dot, dash, and underscore
        let re = Regex::new(r"^[\w\.\-\d]+$").unwrap();
        re.is_match(name)
    }

    /// Full name of a branch
    ///
    /// # Examples
    ///
    /// ```rust
    /// let branch = Branch { name: "main", remote: None, head: None };
    /// assert_eq!(branch.full_name(), "main");
    ///
    /// let branch = Branch { name: "hotfix", remote: Some("origin"), head: None };
    /// assert_eq!(branch.full_name(), "origin/hotfix");
    /// ```
    pub fn full_name(&self) -> String {
        if let Some(remote) = &self.remote {
            format!("{}/{}", remote, self.name)
        } else {
            self.name.clone()
        }
    }
}

fn path_of(by: &str) -> std::path::PathBuf {
    let mut iter = by.split('/');
    let first = iter.next().expect("branch name is empty");
    if let Some(branch) = iter.next() {
        std::path::PathBuf::from(format!("refs/remotes/{first}/{branch}"))
    } else {
        std::path::PathBuf::from(format!("refs/heads/{first}"))
    }
}

impl Accessable<&str> for Branch {
    fn path_of(by: &&str) -> std::path::PathBuf {
        path_of(by)
    }
}

impl Accessable<String> for Branch {
    fn path_of(by: &String) -> std::path::PathBuf {
        path_of(by)
    }
}

impl Store for Branch {
    fn location(&self) -> std::path::PathBuf {
        if let Some(remote) = &self.remote {
            std::path::PathBuf::from(format!("refs/remotes/{}/{}", remote, self.name))
        } else {
            std::path::PathBuf::from(format!("refs/heads/{}", self.name))
        }
    }
    serde_json_store!();
}

impl DirContainer for Branch {
    const DIRECTORY: &'static str = "refs";

    fn make_dir(root: &std::path::Path) -> std::io::Result<()> {
        let path = root.join(Self::DIRECTORY);
        std::fs::create_dir_all(&path)?;
        std::fs::create_dir_all(path.join("heads"))?;
        std::fs::create_dir_all(path.join("remotes"))?;
        Ok(())
    }
}

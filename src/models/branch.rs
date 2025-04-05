//! Branch of the repository
use super::object::ObjectSha1;
use crate::traits::{Accessable, Store};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub remote: Option<String>,
    pub name: String,
    pub head: Option<ObjectSha1>,
}

impl Accessable<String> for Branch {
    fn path_of(by: &String) -> std::path::PathBuf {
        let mut iter = by.split('/');
        if let Some(remote) = iter.next() {
            return std::path::PathBuf::from(format!(
                "refs/remotes/{}/{}",
                remote,
                iter.next().expect("branch name is empty")
            ));
        } else {
            std::path::PathBuf::from(format!("refs/heads/{}", by))
        }
    }
}

impl Store for Branch {
    fn loaction(&self) -> std::path::PathBuf {
        if let Some(remote) = &self.remote {
            std::path::PathBuf::from(format!("refs/remotes/{}/{}", remote, self.name))
        } else {
            std::path::PathBuf::from(format!("refs/heads/{}", self.name))
        }
    }
}

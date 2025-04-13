//! Branch of the repository
use super::object::ObjectSha1;
use crate::{
    serde_json_store,
    traits::{Accessable, DirContainer, Store},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub remote: Option<String>,
    pub name: String,
    pub head: Option<ObjectSha1>,
}

impl Branch {
    pub fn validate_name(name: &str) -> bool {
        //  use regex to match as just include alnum, dot, dash, and underscore
        let re = Regex::new(r"^[\w\.\-\d]+$").unwrap();
        re.is_match(name)
    }
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
        std::path::PathBuf::from(format!("refs/remotes/{}/{}", first, branch))
    } else {
        std::path::PathBuf::from(format!("refs/heads/{}", first))
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

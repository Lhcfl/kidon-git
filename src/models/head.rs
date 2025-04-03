//! A pointer to the currently active branch of the context (repository, remote, etc.)

use crate::traits::{SerDe, Store};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum HeadKind {
    Local,
    Remote(String),
}

#[derive(Debug)]
pub struct Head {
    pub kind: HeadKind,
    pub name: String,
}

impl SerDe for Head {
    fn serialize(&self) -> Vec<u8> {
        format!(
            "ref: refs/{}/{}",
            match &self.kind {
                HeadKind::Local => "heads".to_string(),
                HeadKind::Remote(remote) => format!("remotes/{}", remote),
            },
            self.name
        )
        .into_bytes()
    }

    fn deserialize(data: impl Into<Vec<u8>>) -> Result<Self, String> {
        let str = String::from_utf8(data.into())
            .map_err(|e| format!("Cannot parse data as utf-8 string: {e}"))?;
        Err("unimplemented".to_string())
    }
}

impl Store for Head {
    fn loaction(&self) -> PathBuf {
        Path::new("HEAD").to_path_buf()
    }
}

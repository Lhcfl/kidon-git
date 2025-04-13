use crate::{
    models::{
        branch::Branch,
        repo::{Repository, WithRepo},
    },
    traits::{Accessable, DirContainer},
};
use std::io;

pub enum BranchCreationError {
    AlreadyExists,
    InvalidName,
    IoError(io::Error),
}
impl From<io::Error> for BranchCreationError {
    fn from(err: io::Error) -> Self {
        BranchCreationError::IoError(err)
    }
}
impl From<BranchCreationError> for anyhow::Error {
    fn from(err: BranchCreationError) -> Self {
        match err {
            BranchCreationError::AlreadyExists => anyhow::anyhow!("branch already exists"),
            BranchCreationError::InvalidName => anyhow::anyhow!("invalid branch name"),
            BranchCreationError::IoError(err) => err.into(),
        }
    }
}
pub trait BranchService {
    fn list_branch(&self) -> io::Result<Vec<String>>;
    fn create_branch(&self, branch_name: &str)
    -> Result<WithRepo<'_, Branch>, BranchCreationError>;
}

impl BranchService for Repository {
    /// list branch names, including remote branches, by a vector of strings
    fn list_branch(&self) -> io::Result<Vec<String>> {
        let mut branches = Vec::new();
        for entry in std::fs::read_dir(self.root.join(Branch::DIRECTORY).join("heads"))? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let branch_name = entry.file_name().into_string().map_err(|s| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("branch name {s:?} is not valid"),
                    )
                })?;
                branches.push(branch_name);
            }
        }

        for entry in std::fs::read_dir(self.root.join(Branch::DIRECTORY).join("remotes"))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let remote_name = entry.file_name().into_string().map_err(|s| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("remote name {s:?} is not valid"),
                    )
                })?;
                for branch in std::fs::read_dir(entry.path())? {
                    let branch = branch?;
                    if branch.file_type()?.is_file() {
                        let branch_name = branch.file_name().into_string().map_err(|s| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("branch name {s:?} is not valid"),
                            )
                        })?;
                        branches.push(format!("{remote_name}/{branch_name}"));
                    }
                }
            }
        }

        Ok(branches)
    }

    /// Create a new branch with the given name based on the current branch
    fn create_branch(&self, name: &str) -> Result<WithRepo<'_, Branch>, BranchCreationError> {
        Branch::validate_name(name)
            .then_some(())
            .ok_or(BranchCreationError::InvalidName)?;
        let Err(_) = self.wrap(Branch::accessor(&name)).load() else {
            return Err(BranchCreationError::AlreadyExists);
        };
        let wrap = self.wrap(Branch {
            remote: None,
            name: name.to_string(),
            head: self.head().branch().load()?.unwrap().head,
        });
        wrap.save()?;
        Ok(wrap)
    }
}

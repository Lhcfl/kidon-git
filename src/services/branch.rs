use crate::services::dump_tree::DumpTreeService;
use crate::models::{
        Accessible, DirContainer,
        branch::{Branch, EMPTY_BRANCH_HEAD_PLACEHOLDER},
        repo::{Repository, WithRepo},
        stage::Stage,
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
    fn load_branch<'a>(&'a self, name: &str) -> io::Result<WithRepo<'a, Branch>>;
    fn list_branch(&self) -> io::Result<Vec<String>>;
    fn create_branch(&self, branch_name: &str)
    -> Result<WithRepo<'_, Branch>, BranchCreationError>;
    fn delete_branch(&self, branch_name: &str) -> io::Result<()>;
    fn branch_exists(&self, branch_name: &str) -> io::Result<bool>;
    fn checkout_branch(&mut self, branch_name: &str, dry: bool) -> io::Result<()>;
}

impl BranchService for Repository {
    /// load a branch by its name
    fn load_branch<'a>(&'a self, name: &str) -> io::Result<WithRepo<'a, Branch>> {
        self.wrap(Branch::accessor(&name)).load()
    }

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

        let Ok(current_branch) = self.head().load_branch() else {
            return Ok(self.wrap(Branch {
                remote: None,
                name: name.to_string(),
                head: EMPTY_BRANCH_HEAD_PLACEHOLDER.into(),
            }));
        };

        let wrap = self.wrap(Branch {
            remote: None,
            name: name.to_string(),
            head: current_branch.unwrap().head,
        });

        wrap.save()?;
        Ok(wrap)
    }

    fn delete_branch(&self, name: &str) -> io::Result<()> {
        let branch = self.wrap(Branch::accessor(&name));
        let branch = branch.load()?;
        if branch.full_name() == self.head().branch_name {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot delete current branch",
            ));
        }
        branch.remove()?;
        Ok(())
    }

    fn branch_exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.list_branch()?.iter().any(|b| b == name))
    }

    fn checkout_branch(&mut self, name: &str, dry: bool) -> io::Result<()> {
        // Step 0: if is dry checkout
        if dry && self.list_branch().unwrap().is_empty() {
            // Step 0-8: Update HEAD to point to the new branch, no need to modify anything.
            let head = self.head_mut();
            head.branch_name = name.into();
            self.save_head()?;
            return Ok(());
        }
        // Step 1: Check if the branch exists
        if !self.branch_exists(name)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("branch '{name}' does not exist"),
            ));
        }

        // if dry checkout, just update the HEAD
        let Ok(_) = self.head().load_branch() else {
            let head = self.head_mut();
            head.branch_name = name.to_string();
            self.save_head()?;
            return Ok(());
        };

        let target_branch = self.load_branch(name)?;
        let target_commit = target_branch.get_current_commit()?;
        let target_tree = target_commit.get_tree()?;

        self.dump_tree(&target_tree)?;

        // save the target tree to the stage
        target_tree.map(Stage).save()?;

        // Step 8: Update HEAD to point to the new branch
        let head = self.head_mut();
        head.branch_name = name.into();
        self.save_head()?;
        Ok(())
    }
}

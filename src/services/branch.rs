use crate::{
    models::{
        branch::{Branch, EMPTY_BRANCH_HEAD_PLACEHOLDER}, object::Object, repo::{Repository, WithRepo}, stage::Stage, Accessible, DirContainer
    }, services::{tree::{compare_trees, ComparedKind}}
};
use std::io;
use crate::models::commit::Commit;

impl WithRepo<'_, Branch> {
    pub fn get_current_commit(&self) -> io::Result<WithRepo<Commit>> {
        let sha1 = &self.head;
        let obj = self.wrap(Object::accessor(sha1)).load()?;
        Ok(obj.map(|o| o.cast_commit()))
    }
}

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
    fn delete_branch(&self, branch_name: &str) -> io::Result<()>;
    fn branch_exists(&self, branch_name: &str) -> io::Result<bool>;
    fn checkout_branch(&mut self, branch_name: &str, dry: bool) -> io::Result<()>;
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
        if dry && self.list_branch().unwrap().len()==0 {
            // Step 0-8: Update HEAD to point to the new branch, no need to modify anything.
            let head = self.head_mut();
            head.branch_name = name.into();
            self.save_head()?;
            return Ok(())
        }
        // Step 1: Check if the branch exists
        if !self.branch_exists(name)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("branch '{name}' does not exist"),
            ));
        }

        let Ok(current_branch) = self.head().load_branch() else {
            let head = self.head_mut();
            head.branch_name = name.to_string();
            self.save_head()?;
            return Ok(());
        };

        // Step 2: Load the branch
        let target_branch = self.wrap(Branch::accessor(&name)).load()?;

        // Step 4: Load the target branch's tree
        let target_commit = target_branch.get_current_commit()?;
        let target_tree = target_commit.get_tree()?;
        // Step 5: Load the current branch's tree
        let current_commit = current_branch.get_current_commit()?;
        let current_tree = current_commit.get_tree()?;

        // Step 6: Compare the trees
        let changes = compare_trees(&current_tree, &target_tree)?;

        // Step 7: Apply changes to the working directory
        for change in changes {
            match change.kind {
                ComparedKind::Added | ComparedKind::Modified => {
                    // Write new or modified files
                    let blob = self
                        .wrap(Object::accessor(&change.line.sha1))
                        .load()?.clone()
                        .cast_blob();
                    // Emmm.. assuming workign dir is .git's parent @lhcfl maybe add pwd root in repo?
                    let path = self.working_dir().join(&change.line.name);

                    // Ensure parent directories exist
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    // Write the file
                    std::fs::write(&path, blob.as_bytes())?;
                }
                ComparedKind::Deleted => {
                    // Remove deleted files
                    let path = self.working_dir().join(&change.line.name);

                    if path.is_file() || path.is_symlink() {
                        std::fs::remove_file(&path)?;
                    } else if path.is_dir() {
                        std::fs::remove_dir_all(&path)?;
                    }
                }
            }
        }
        
        // save the target tree to the stage
        target_tree.map(Stage).save()?;

        // Step 8: Update HEAD to point to the new branch
        let head = self.head_mut();
        head.branch_name = name.into();
        self.save_head()?;
        Ok(())
    }
}


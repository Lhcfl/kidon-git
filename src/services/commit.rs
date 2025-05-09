use crate::{
    models::{
        branch::Branch,
        commit::{Commit, CommitBuilder},
        object::{Object, Sha1Able},
        repo::Repository,
    },
    services::tree::compare_trees,
    traits::Accessable,
};

use super::tree::{ComparedKind, ComparedLine};

pub struct CommitCreationInfo {
    pub compared: Option<Vec<ComparedLine>>,
    pub commit_sha1: String,
    pub branch: Branch,
}

pub enum CommitCreateResult {
    Success(CommitCreationInfo),
    NothingToCommit,
}

pub trait CommitService {
    fn create_commit(&self, message: &str) -> anyhow::Result<CommitCreateResult>;
}

impl CommitService for Repository {
    fn create_commit(&self, message: &str) -> anyhow::Result<CommitCreateResult> {
        let branch = self.head().branch().load()?;
        let message = message.to_owned();

        // Step 2: Generate tree object from the stage
        let stage = self.stage()?;
        let tree = stage.map(|s| s.0);

        // Step 3: Compare the tree with the current HEAD, to check if working tree clean
        let compared = if let Some(current_sha1) = branch.head.as_ref() {
            let current_commit = self.wrap(Object::accessor(current_sha1)).load()?;
            let current_tree_sha1 = current_commit.unwrap().cast_commit().tree;
            let current_tree = self
                .wrap(Object::accessor(&current_tree_sha1))
                .load()?
                .map(|t| t.cast_tree());

            let compared = compare_trees(&current_tree, &tree)?;
            if compared.is_empty() {
                return Ok(CommitCreateResult::NothingToCommit);
            }
            Some(compared)
        } else {
            None
        };

        // Step 4: Create commit object
        let tree = tree.map(Object::Tree);
        tree.save()?;
        let commit = Commit::new(CommitBuilder {
            tree: tree.sha1().into(),
            parent: branch.head.clone(),
            message: message.to_string(),
        });

        let commit_sha1 = commit.sha1();
        self.wrap(Object::Commit(commit)).save()?;

        // Step 5: Update branch HEAD
        let mut branch_cloned = branch.cloned();
        branch_cloned.head = Some(commit_sha1.clone().into());
        branch_cloned.save()?;

        return Ok(CommitCreateResult::Success(CommitCreationInfo {
            compared,
            commit_sha1,
            branch: branch.unwrap(),
        }));
    }
}

use std::ops::Not;

use clap::Args;
use colored::Colorize;

use crate::{
    console_output,
    models::{Accessible, object::Object, repo::Repository, tree::Tree},
    services::tree::{ComparedKind, compare_trees},
};

use super::Exec;

#[derive(Debug, Args)]
pub struct Status {}

impl Exec for Status {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;
        let branch = repo.head().load_branch();

        console_output!("On branch {}", repo.head().branch_name);

        let working_tree = repo.working_tree()?;
        let stage_tree = repo.stage()?.map(|s| s.0);
        let head_tree = if let Ok(sha1) = branch.map(|b| b.unwrap().head) {
            let head_commit = repo
                .wrap(Object::accessor(&sha1))
                .load()?
                .map(|c| c.cast_commit());

            repo.wrap(Object::accessor(&head_commit.tree))
                .load()?
                .map(|t| t.cast_tree())
        } else {
            console_output!("No commits yet\n");
            repo.wrap(Tree::empty())
        };

        let mut staging_changes = compare_trees(&head_tree, &stage_tree)?;
        staging_changes.sort_by(|a, b| a.line.name.cmp(&b.line.name));
        let mut working_changes = compare_trees(&stage_tree, &working_tree)?;
        working_changes.sort_by(|a, b| a.line.name.cmp(&b.line.name));

        if staging_changes.is_empty().not() {
            console_output!(
                "
Changes to be committed:
  (use \"git restore --staged <file>...\" to unstage)"
            );
            for diff in &staging_changes {
                match diff.kind {
                    ComparedKind::Modified => {
                        console_output!("{}", diff.to_string().yellow());
                    }
                    ComparedKind::Deleted => {
                        console_output!("{}", diff.to_string().red());
                    }
                    ComparedKind::Added => {
                        console_output!("{}", diff.to_string().green());
                    }
                }
            }
        }

        let changes_not_staged_for_commit = working_changes
            .iter()
            .filter(|x| x.kind != ComparedKind::Added)
            .collect::<Vec<_>>();

        if changes_not_staged_for_commit.is_empty().not() {
            console_output!(
                "
Changes not staged for commit:
  (use \"git add <file>...\" to update what will be committed)
  (use \"git restore <file>...\" to discard changes in working directory)"
            );
            for diff in changes_not_staged_for_commit {
                match diff.kind {
                    ComparedKind::Modified => {
                        console_output!("{}", diff.to_string().yellow());
                    }
                    ComparedKind::Deleted => {
                        console_output!("{}", diff.to_string().red());
                    }
                    ComparedKind::Added => {
                        console_output!("{}", diff.to_string().green());
                    }
                }
            }
        }

        let untracked = working_changes
            .iter()
            .filter(|x| x.kind == ComparedKind::Added)
            .collect::<Vec<_>>();

        if untracked.is_empty().not() {
            console_output!(
                "
Untracked files:
  (use \"git add <file>...\" to include in what will be committed)"
            );
            for diff in untracked {
                console_output!("        {}", diff.line.name.green());
            }
        }

        if working_changes.is_empty() && staging_changes.is_empty() {
            console_output!("nothing to commit, working tree clean");
        }

        Ok(())
    }
}

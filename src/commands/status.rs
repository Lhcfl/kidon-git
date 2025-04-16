use std::ops::Not;

use clap::Args;
use colored::Colorize;

use crate::{
    models::{object::Object, repo::Repository},
    services::tree::{ComparedKind, compare_trees},
    traits::Accessable,
};

use super::Exec;

#[derive(Debug, Args)]
pub struct Status {}

impl Exec for Status {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;
        let branch = repo.head().branch().load()?;

        println!("On branch {}", branch.name);

        let stage_tree = repo.stage()?.map(|s| s.0);
        let staging_change = if let Some(sha1) = &branch.head {
            let head_commit = repo
                .wrap(Object::accessor(sha1))
                .load()?
                .map(|c| c.cast_commit());
            let head_tree = repo
                .wrap(Object::accessor(&head_commit.tree))
                .load()?
                .map(|t| t.cast_tree());

            let mut res = compare_trees(&head_tree, &stage_tree)?;
            res.sort_by(|a, b| a.line.name.cmp(&b.line.name));
            res
        } else {
            println!("\nNo commits yet");
            Vec::new()
        };

        if staging_change.is_empty().not() {
            println!(
                "
Changes to be committed:
  (use \"git restore --staged <file>...\" to unstage)"
            );
            for diff in &staging_change {
                match diff.kind {
                    ComparedKind::Modified => {
                        println!("{}", diff.to_string().yellow());
                    }
                    ComparedKind::Deleted => {
                        println!("{}", diff.to_string().red());
                    }
                    ComparedKind::Added => {
                        println!("{}", diff.to_string().green());
                    }
                }
            }
        }

        let working_tree = repo.working_tree()?;
        let mut working_change = compare_trees(&stage_tree, &working_tree)?;

        working_change.sort_by(|a, b| a.line.name.cmp(&b.line.name));

        let changes_not_staged_for_commit = working_change
            .iter()
            .filter(|x| x.kind != ComparedKind::Added)
            .collect::<Vec<_>>();

        if changes_not_staged_for_commit.is_empty().not() {
            println!(
                "
Changes not staged for commit:
  (use \"git add <file>...\" to update what will be committed)
  (use \"git restore <file>...\" to discard changes in working directory)"
            );
            for diff in changes_not_staged_for_commit {
                match diff.kind {
                    ComparedKind::Modified => {
                        println!("{}", diff.to_string().yellow());
                    }
                    ComparedKind::Deleted => {
                        println!("{}", diff.to_string().red());
                    }
                    ComparedKind::Added => {
                        println!("{}", diff.to_string().green());
                    }
                }
            }
        }

        let untracked = working_change
            .iter()
            .filter(|x| x.kind == ComparedKind::Added)
            .collect::<Vec<_>>();

        if untracked.is_empty().not() {
            println!(
                "
Untracked files:
  (use \"git add <file>...\" to include in what will be committed)"
            );
            for diff in untracked {
                println!("        {}", diff.line.name.green());
            }
        }

        if working_change.is_empty() && staging_change.is_empty() {
            println!("nothing to commit, working tree clean");
        }

        Ok(())
    }
}

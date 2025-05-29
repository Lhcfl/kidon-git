use crate::{
    console_output, models::repo::Repository, oj_output, services::commit::{CommitCreateResult, CommitService}
};
use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Commit {
    /// commit message
    #[arg(short, long)]
    pub message: Option<String>,
}

impl Exec for Commit {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = Repository::load()?;

        let Some(message) = &self.message else {
            anyhow::bail!("commit message is required");
        };

        let res = repo.create_commit(message)?;

        match res {
            CommitCreateResult::Success(res) => {
                let tip = if res.is_root {
                    " (root-commit) "
                } else {
                    " "
                };

                console_output!(
                    "[{}{tip}{}] {message}",
                    res.branch_name,
                    &res.commit_sha1[0..7]
                );

                // Log commit status
                if let Some(compared) = res.compared {
                    console_output!("{} file changed", compared.len());
                    for line in compared {
                        console_output!("{line}");
                    }
                }

                oj_output!("{}", res.commit_sha1);
            }
            CommitCreateResult::NothingToCommit => {
                console_output!("nothing to commit, working tree clean");
            }
        }

        Ok(())
    }
}

use super::Exec;
use crate::models::{Accessible, branch::Branch, repo};
use crate::services::merge::MergeService;
use clap::Args;

#[derive(Debug, Args)]
pub struct Merge {
    branch: String,
}

impl Exec for Merge {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = repo::Repository::load()?;
        let Ok(branch) = repo.wrap(Branch::accessor(&self.branch.as_str())).load() else {
            anyhow::bail!("branch {} not found", self.branch);
        };

        repo.merge(branch.unwrap())?;
        Ok(())
    }
}

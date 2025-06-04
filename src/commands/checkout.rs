use clap::Args;

use super::Exec;
use crate::{models::repo::Repository, services::branch::BranchService};

#[derive(Debug, Args)]
pub struct Checkout {
    /// create branch first. if the branch already exists, will exit with an error
    #[arg(short('b'), long)]
    pub create: bool,
    /// the branch that will checkout to
    pub branch: String,
}

impl Exec for Checkout {
    fn exec(&self) -> anyhow::Result<()> {
        let mut repo = Repository::load()?;
        let branch_name = &self.branch;

        if self.create {
            repo.create_branch(branch_name)?;
        }

        // Check if the branch exists
        if repo.list_branch().unwrap().len()!=0 && !repo.branch_exists(branch_name)? {
            anyhow::bail!("pathspec '{branch_name}' did not match any file(s) known to git"); 
        }

        // Switch to the branch
        repo.checkout_branch(branch_name, self.create)?;

        // console_output!("Switched to branch '{}'", branch_name);
        Ok(())
    }
}

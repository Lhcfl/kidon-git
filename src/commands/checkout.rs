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
        let repo = Repository::load()?;
        let branch_name = &self.branch;
        if self.create {
            repo.create_branch(&branch_name)?;
        }
        // change branch to branch_name
        // TODO
        // TODO @leonard 可以把 services/branch 的 creation 用在这里
        panic!("checkout is not implemented")
    }
}

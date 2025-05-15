use super::Exec;
use crate::{models::repo::Repository, services::branch::BranchService};
use clap::Args;
use colored::Colorize;

#[derive(Debug, Args)]
pub struct Branch {
    /// delete branch
    #[arg(short, long)]
    pub delete: bool,

    pub name: Option<String>,
}

fn list_branch() -> anyhow::Result<()> {
    let repo = Repository::load()?;
    let branches = repo.list_branch()?;
    for branch in branches {
        if repo.head().branch_name == branch {
            println!("* {}", branch.green());
        } else {
            println!("  {branch}");
        }
    }
    Ok(())
}

impl Exec for Branch {
    fn exec(&self) -> anyhow::Result<()> {
        if self.name.is_none() {
            if self.delete {
                return Err(anyhow::anyhow!("branch name required"));
            } else {
                return list_branch();
            }
        }

        let repo= Repository::load()?;
        let branch_name = self.name.as_ref().unwrap();
        if self.delete {
            // delete branch
            repo.delete_branch(branch_name)?;
            Ok(())
        } else {
            // create branch
            repo.create_branch(branch_name)?;
            Ok(())
        }
    }
}

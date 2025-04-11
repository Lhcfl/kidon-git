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
            println!("  {}", branch);
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

        if self.delete {
            // delete branch
            panic!("branch delete is not implemented")
            // TODO @leonard 把 branch 的创建和删除作为一个实现给 repo 的 trait
            // 放在 services/branch.rs 的 impl BranchService for Repository 里
        } else {
            // create branch
            panic!("branch creation is not implemented")
            // TODO @leonard
        }
    }
}

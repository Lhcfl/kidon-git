use clap::Args;

use crate::services;

use super::Exec;

#[derive(Debug, Args)]
pub struct Branch {
    /// delete branch
    #[arg(short, long)]
    pub delete: bool,

    pub name: Option<String>,
}

impl Exec for Branch {
    fn exec(&self) -> anyhow::Result<()> {
        let repo = services::repo::ensure_exists_or_log()?;
        println!("{}", repo.head().branch_name);
        panic!("branch is not implemented")
    }
}

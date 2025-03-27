use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Branch {
    /// delete branch
    #[arg(short, long)]
    pub delete: bool,
}

impl Exec for Branch {
    fn exec(&self) {
        panic!("branch is not implemented")
    }
}

use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Merge {
    branch: String,
}

impl Exec for Merge {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("merge is not implemented")
    }
}

use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Merge {}

impl Exec for Merge {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("merge is not implemented")
    }
}

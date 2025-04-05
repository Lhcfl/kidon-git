use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Pull {}

impl Exec for Pull {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("init is not implemented")
    }
}

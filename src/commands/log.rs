use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Log {}

impl Exec for Log {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("init is not implemented")
    }
}

use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Checkout {
    /// delete branch
    pub branch: String,
}

impl Exec for Checkout {
    fn exec(&self) -> anyhow::Result<()> {
        panic!("checkout is not implemented")
    }
}

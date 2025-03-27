use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Init {}

impl Exec for Init {
    fn exec(&self) {
        panic!("init is not implemented")
    }
}

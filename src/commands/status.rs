use clap::Args;

use super::Exec;

#[derive(Debug, Args)]
pub struct Status {}

impl Exec for Status {
    fn exec(&self) {
        panic!("init is not implemented")
    }
}

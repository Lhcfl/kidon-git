use clap::Subcommand;

mod add;
mod branch;
mod checkout;
mod commit;
mod init;
mod merge;
mod rm;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Commit this
    Commit(commit::Commit),
    /// Add files to buffer
    Add(add::Add),
    Init(init::Init),
    Branch(branch::Branch),
    Checkout(checkout::Checkout),
    Merge(merge::Merge),
    Rm(rm::Rm),
    Fetch,
    Pull,
    Push,
}

pub trait Exec {
    /// execute the command
    fn exec(&self);
}

impl Commands {
    pub fn show(&self) {
        println!("{self:?}");
    }
}

impl Exec for Commands {
    fn exec(&self) {
        use Commands::*;
        match self {
            Commit(data) => data.exec(),
            Add(data) => data.exec(),
            Init(data) => data.exec(),
            Branch(data) => data.exec(),
            Checkout(data) => data.exec(),
            Merge(data) => data.exec(),
            Rm(data) => data.exec(),
            Fetch => {
                panic!("Not Implemented");
            }
            Pull => {
                panic!("Not Implemented");
            }
            Push => {
                panic!("Not Implemented");
            }
        }
    }
}

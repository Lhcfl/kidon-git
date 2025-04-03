use clap::Args;

use crate::services::{self, repo};

use super::Exec;

#[derive(Debug, Args)]
pub struct Init {}

impl Exec for Init {
    fn exec(&self) {
        match services::repo::load() {
            Ok(repo) => {
                println!(
                    "the git repository exists in {}",
                    repo.path.to_string_lossy()
                );
                return;
            }
            Err(repo::RepositoryInitError::NotADirectory) => {
                println!("the .git folder is not a directory");
                return;
            }
            Err(repo::RepositoryInitError::UnknownError(e)) => {
                println!("error: unknown error: {e}");
                return;
            }
            Err(repo::RepositoryInitError::NotInitialized) => {}
        }

        panic!("init is not implemented")
    }
}

use clap::Args;

use crate::models::repo::{Repository, RepositoryInitError};

use super::Exec;

#[derive(Debug, Args)]
pub struct Init {}

impl Exec for Init {
    fn exec(&self) {
        match Repository::load() {
            Ok(repo) => {
                println!(
                    "the git repository exists in {}",
                    repo.root.to_string_lossy()
                );
                return;
            }
            Err(RepositoryInitError::NotInitialized) => {
                // noting to do
            }
            Err(e) => {
                println!("error: {e:?}");
                return;
            }
        }

        match Repository::init() {
            Ok(repo) => {
                println!(
                    "successfully initialized git repo in {}",
                    repo.root.to_string_lossy()
                );
            }
            Err(e) => {
                println!("error: failed to initialize git repo: {e:?}");
            }
        }
    }
}

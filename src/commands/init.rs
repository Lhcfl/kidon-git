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
            Err(repo::RepositoryInitError::NotInitialized) => {
                // noting to do
            }
            Err(e) => {
                println!("error: {e:?}");
                return;
            }
        }

        match services::repo::init() {
            Ok(repo) => {
                println!(
                    "successfully initialized git repo in {}",
                    repo.path.to_string_lossy()
                );
            }
            Err(e) => {
                println!("error: failed to initialize git repo: {e:?}");
            }
        }
    }
}

use crate::models::repo::{Repository, RepositoryInitError};

pub fn ensure_exists_or_log() -> Result<Repository, RepositoryInitError> {
    match Repository::load() {
        Ok(r) => Ok(r),
        Err(RepositoryInitError::NotInitialized) => {
            println!("fatal: not a git repository (or any of the parent directories)");
            Err(RepositoryInitError::NotInitialized)
        }
        Err(e) => {
            println!("fatal: not a git repository, or the git repository is broken");
            Err(e)
        }
    }
}

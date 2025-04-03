use std::path::Path;

const REFS_DIR: &str = "refs";

/// Referance to a commit
pub struct Refs {
    pub sha1: String,
}

impl Refs {
    pub fn new(sha1: String) -> Self {
        Self { sha1 }
    }
}

pub fn init_dir(path: &Path) -> Result<(), std::io::Error> {
    let refs_path = path.join(REFS_DIR);
    std::fs::create_dir_all(refs_path)?;
    Ok(())
}

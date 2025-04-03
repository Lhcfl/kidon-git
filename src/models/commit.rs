use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub timestamp: SystemTime,
    pub message: String,
}

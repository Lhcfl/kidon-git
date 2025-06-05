//! Services of git
//!
//! To reduce complexity and circular dependencies, we put all git specific
//! logic, such as creating branches or merging commits, in services.
//!
//! Methods in services should be independent and single-responsible, easy to
//! combine and reuse.

pub mod branch;
pub mod commit;
pub mod dump_tree;
pub mod merge;
pub mod mut_tree;
pub mod object;
pub mod oj;
pub mod repo;
pub mod stage;
pub mod tree;

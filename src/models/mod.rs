//! Models of git
//!
//! models represent entities that will eventually be stored in the file system
//! (they are [Store](crate::traits::Store)s) or need to be kept in memory at
//! runtime. You should not put specific interactive functions here except
//! loading and saving from disk.

pub mod blob;
pub mod branch;
pub mod commit;
pub mod head;
pub mod ignores;
pub mod object;
pub mod repo;
pub mod stage;
pub mod tree;

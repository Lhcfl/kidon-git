use std::io;

use crate::models::{
    Accessible,
    object::{Object, ObjectSha1},
    repo::{Repository, WithRepo},
};

pub trait ObjectService {
    /// Load an object by its SHA1
    fn load_object<'a>(&'a self, sha1: &ObjectSha1) -> io::Result<WithRepo<'a, Object>>;
}

impl ObjectService for Repository {
    fn load_object<'a>(&'a self, sha1: &ObjectSha1) -> io::Result<WithRepo<'a, Object>> {
        self.wrap(Object::accessor(sha1)).load()
    }
}

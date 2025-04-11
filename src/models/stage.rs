//! stage files of the repo
//! The stage files are used to store the changes that are not yet committed.
//! It should not be saved by [object], since the stage files are not part of the commit.

struct StagedFile {
    context: String,
}

//! A pointer to the currently active branch of the context (repository, remote, etc.)

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::char,
};

use crate::traits::{SerDe, Store};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum HeadKind {
    Local,
    Remote(String),
}

#[derive(Debug, PartialEq)]
pub struct Head {
    pub kind: HeadKind,
    pub branch: String,
}

impl Head {
    fn parse_from_str(str: &str) -> IResult<&str, Head> {
        let (rest, (_, location)) =
            (tag("ref: refs/"), alt((tag("heads/"), tag("remotes/")))).parse(str)?;

        match location {
            "heads/" => Ok((
                "",
                Head {
                    kind: HeadKind::Local,
                    branch: rest.to_string(),
                },
            )),
            "remotes/" => {
                let (branch, (remote, _)) = (take_while(|c| c != '/'), char('/')).parse(rest)?;

                Ok((
                    "",
                    Head {
                        kind: HeadKind::Remote(remote.to_string()),
                        branch: branch.to_string(),
                    },
                ))
            }
            _ => unreachable!(),
        }
    }
}

impl SerDe for Head {
    fn serialize(&self) -> Vec<u8> {
        format!(
            "ref: refs/{}/{}",
            match &self.kind {
                HeadKind::Local => "heads".to_string(),
                HeadKind::Remote(remote) => format!("remotes/{}", remote),
            },
            self.branch
        )
        .into_bytes()
    }

    fn deserialize(data: impl Into<Vec<u8>>) -> Result<Self, String> {
        let str = String::from_utf8(data.into())
            .map_err(|e| format!("Cannot parse data as utf-8 string: {e}"))?;

        let (_, ret) =
            Self::parse_from_str(&str).map_err(|e| format!("Cannot parse data as head: {e}"))?;

        Ok(ret)
    }
}

impl Store for Head {
    fn loaction(&self) -> PathBuf {
        Path::new("HEAD").to_path_buf()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_head() {
        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Local,
                    branch: "main".to_string(),
                }
                .serialize()
            ),
            "ref: refs/heads/main"
        );
        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Local,
                    branch: "awe-a-fe-awef".to_string(),
                }
                .serialize()
            ),
            "ref: refs/heads/awe-a-fe-awef"
        );
        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Local,
                    branch: "中文".to_string(),
                }
                .serialize()
            ),
            "ref: refs/heads/中文"
        );

        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Remote("origin".to_string()),
                    branch: "main".to_string(),
                }
                .serialize()
            ),
            "ref: refs/remotes/origin/main"
        );

        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Remote("origin".to_string()),
                    branch: "awe-a-fe-awef".to_string(),
                }
                .serialize()
            ),
            "ref: refs/remotes/origin/awe-a-fe-awef"
        );

        assert_eq!(
            String::from_utf8_lossy(
                &Head {
                    kind: HeadKind::Remote("origin".to_string()),
                    branch: "中文".to_string(),
                }
                .serialize()
            ),
            "ref: refs/remotes/origin/中文"
        );
    }

    #[test]
    fn test_deserialize_head() {
        assert_eq!(
            Head::deserialize("ref: refs/heads/main".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Local,
                branch: "main".to_string(),
            }
        );
        assert_eq!(
            Head::deserialize("ref: refs/heads/awe-a-fe-awef".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Local,
                branch: "awe-a-fe-awef".to_string(),
            }
        );
        assert_eq!(
            Head::deserialize("ref: refs/heads/中文".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Local,
                branch: "中文".to_string(),
            }
        );
        assert_eq!(
            Head::deserialize("ref: refs/remotes/origin/main".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Remote("origin".to_string()),
                branch: "main".to_string(),
            }
        );
        assert_eq!(
            Head::deserialize("ref: refs/remotes/origin/awe-a-fe-awef".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Remote("origin".to_string()),
                branch: "awe-a-fe-awef".to_string(),
            }
        );
        assert_eq!(
            Head::deserialize("ref: refs/remotes/origin/中文".as_bytes()).unwrap(),
            Head {
                kind: HeadKind::Remote("origin".to_string()),
                branch: "中文".to_string(),
            }
        );
    }
}

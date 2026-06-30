use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh release delete`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDelete {
    pub tag: String,
    pub repo: Option<String>,
    pub cleanup_tag: bool,
    pub yes: bool,
}

impl ReleaseDelete {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            repo: None,
            cleanup_tag: false,
            yes: true,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["release".into(), "delete".into(), self.tag.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_bool(&mut args, "--cleanup-tag", self.cleanup_tag);
        push_bool(&mut args, "--yes", self.yes);
        args
    }
}

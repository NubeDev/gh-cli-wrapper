use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue delete`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDelete {
    pub issue: String,
    pub repo: Option<String>,
    pub yes: bool,
}

impl IssueDelete {
    pub fn new(issue: impl Into<String>) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            yes: true,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "delete".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_bool(&mut args, "--yes", self.yes);
        args
    }
}

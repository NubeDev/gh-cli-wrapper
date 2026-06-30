use std::ffi::OsString;

use crate::args::push_opt;
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue reopen`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueReopen {
    pub issue: String,
    pub repo: Option<String>,
    pub comment: Option<String>,
}

impl IssueReopen {
    pub fn new(issue: impl Into<String>) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            comment: None,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "reopen".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--comment", &self.comment);
        args
    }
}

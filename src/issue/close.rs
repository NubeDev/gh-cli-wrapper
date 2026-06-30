use std::ffi::OsString;

use crate::args::push_opt;
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue close`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueClose {
    pub issue: String,
    pub repo: Option<String>,
    pub comment: Option<String>,
    pub reason: Option<String>,
}

impl IssueClose {
    pub fn new(issue: impl Into<String>) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            comment: None,
            reason: None,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "close".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--comment", &self.comment);
        push_opt(&mut args, "--reason", &self.reason);
        args
    }
}

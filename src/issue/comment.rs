use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue comment`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueComment {
    pub issue: String,
    pub repo: Option<String>,
    pub body: Option<String>,
    pub body_file: Option<String>,
    pub edit_last: bool,
}

impl IssueComment {
    pub fn new(issue: impl Into<String>) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            body: None,
            body_file: None,
            edit_last: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "comment".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--body", &self.body);
        push_opt(&mut args, "--body-file", &self.body_file);
        push_bool(&mut args, "--edit-last", self.edit_last);
        args
    }
}

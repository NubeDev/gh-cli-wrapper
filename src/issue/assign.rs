use std::ffi::OsString;

use crate::args::{push_opt, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Convenience wrapper for assigning users with `gh issue edit --add-assignee`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueAssign {
    pub issue: String,
    pub repo: Option<String>,
    pub assignees: Vec<String>,
}

impl IssueAssign {
    pub fn new(
        issue: impl Into<String>,
        assignees: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            assignees: assignees.into_iter().map(Into::into).collect(),
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "edit".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_repeated(&mut args, "--add-assignee", &self.assignees);
        args
    }
}

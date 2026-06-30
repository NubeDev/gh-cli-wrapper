use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh pr close`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrClose {
    pub pull_request: String,
    pub repo: Option<String>,
    pub comment: Option<String>,
    pub delete_branch: bool,
}

impl PrClose {
    pub fn new(pull_request: impl Into<String>) -> Self {
        Self {
            pull_request: pull_request.into(),
            repo: None,
            comment: None,
            delete_branch: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec![
            "pr".into(),
            "close".into(),
            self.pull_request.clone().into(),
        ];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--comment", &self.comment);
        push_bool(&mut args, "--delete-branch", self.delete_branch);
        args
    }
}

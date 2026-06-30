use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh pr checkout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrCheckout {
    pub pull_request: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub detach: bool,
    pub force: bool,
    pub recurse_submodules: bool,
}

impl PrCheckout {
    pub fn new(pull_request: impl Into<String>) -> Self {
        Self {
            pull_request: pull_request.into(),
            repo: None,
            branch: None,
            detach: false,
            force: false,
            recurse_submodules: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec![
            "pr".into(),
            "checkout".into(),
            self.pull_request.clone().into(),
        ];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--branch", &self.branch);
        push_bool(&mut args, "--detach", self.detach);
        push_bool(&mut args, "--force", self.force);
        push_bool(&mut args, "--recurse-submodules", self.recurse_submodules);
        args
    }
}

use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Merge method accepted by `gh pr merge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrMergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl PrMergeMethod {
    fn flag(self) -> &'static str {
        match self {
            Self::Merge => "--merge",
            Self::Squash => "--squash",
            Self::Rebase => "--rebase",
        }
    }
}

/// Options for `gh pr merge`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrMerge {
    pub pull_request: String,
    pub repo: Option<String>,
    pub method: PrMergeMethod,
    pub delete_branch: bool,
    pub auto: bool,
    pub subject: Option<String>,
    pub body: Option<String>,
}

impl PrMerge {
    pub fn new(pull_request: impl Into<String>, method: PrMergeMethod) -> Self {
        Self {
            pull_request: pull_request.into(),
            repo: None,
            method,
            delete_branch: false,
            auto: false,
            subject: None,
            body: None,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec![
            "pr".into(),
            "merge".into(),
            self.pull_request.clone().into(),
        ];
        push_opt(&mut args, "--repo", &self.repo);
        args.push(self.method.flag().into());
        push_bool(&mut args, "--delete-branch", self.delete_branch);
        push_bool(&mut args, "--auto", self.auto);
        push_opt(&mut args, "--subject", &self.subject);
        push_opt(&mut args, "--body", &self.body);
        args
    }
}

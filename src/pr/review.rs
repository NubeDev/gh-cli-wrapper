use std::ffi::OsString;

use crate::args::push_opt;
use crate::{Error, Gh, GhOutput};

/// Review action accepted by `gh pr review`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrReviewAction {
    Approve,
    RequestChanges,
    Comment,
}

impl PrReviewAction {
    fn flag(self) -> &'static str {
        match self {
            Self::Approve => "--approve",
            Self::RequestChanges => "--request-changes",
            Self::Comment => "--comment",
        }
    }
}

/// Options for `gh pr review`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrReview {
    pub pull_request: String,
    pub repo: Option<String>,
    pub action: PrReviewAction,
    pub body: Option<String>,
}

impl PrReview {
    pub fn new(pull_request: impl Into<String>, action: PrReviewAction) -> Self {
        Self {
            pull_request: pull_request.into(),
            repo: None,
            action,
            body: None,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec![
            "pr".into(),
            "review".into(),
            self.pull_request.clone().into(),
        ];
        push_opt(&mut args, "--repo", &self.repo);
        args.push(self.action.flag().into());
        push_opt(&mut args, "--body", &self.body);
        args
    }
}

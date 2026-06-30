use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh pr view`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrView {
    pub pull_request: String,
    pub repo: Option<String>,
    pub comments: bool,
    pub json_fields: Vec<String>,
    pub web: bool,
}

impl PrView {
    pub fn new(pull_request: impl Into<String>) -> Self {
        Self {
            pull_request: pull_request.into(),
            repo: None,
            comments: false,
            json_fields: Vec::new(),
            web: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn json<T>(&self, gh: &Gh) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        gh.json(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["pr".into(), "view".into(), self.pull_request.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_bool(&mut args, "--comments", self.comments);
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        push_bool(&mut args, "--web", self.web);
        args
    }
}

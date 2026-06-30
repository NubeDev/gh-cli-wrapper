use std::ffi::OsString;

use crate::args::{push_bool, push_opt, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Options for `gh pr create`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrCreate {
    pub repo: Option<String>,
    pub base: Option<String>,
    pub head: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub assignees: Vec<String>,
    pub reviewers: Vec<String>,
    pub labels: Vec<String>,
    pub draft: bool,
    pub fill: bool,
    pub web: bool,
}

impl PrCreate {
    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["pr".into(), "create".into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--base", &self.base);
        push_opt(&mut args, "--head", &self.head);
        push_opt(&mut args, "--title", &self.title);
        push_opt(&mut args, "--body", &self.body);
        push_repeated(&mut args, "--assignee", &self.assignees);
        push_repeated(&mut args, "--reviewer", &self.reviewers);
        push_repeated(&mut args, "--label", &self.labels);
        push_bool(&mut args, "--draft", self.draft);
        push_bool(&mut args, "--fill", self.fill);
        push_bool(&mut args, "--web", self.web);
        args
    }
}

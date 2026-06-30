use std::ffi::OsString;

use crate::args::{push_bool, push_opt, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue create`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IssueCreate {
    pub repo: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub projects: Vec<String>,
    pub recover: Option<String>,
    pub web: bool,
}

impl IssueCreate {
    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "create".into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--title", &self.title);
        push_opt(&mut args, "--body", &self.body);
        push_repeated(&mut args, "--assignee", &self.assignees);
        push_repeated(&mut args, "--label", &self.labels);
        push_opt(&mut args, "--milestone", &self.milestone);
        push_repeated(&mut args, "--project", &self.projects);
        push_opt(&mut args, "--recover", &self.recover);
        push_bool(&mut args, "--web", self.web);
        args
    }
}

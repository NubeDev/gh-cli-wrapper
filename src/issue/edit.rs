use std::ffi::OsString;

use crate::args::{push_opt, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Options for `gh issue edit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueEdit {
    pub issue: String,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub add_assignees: Vec<String>,
    pub remove_assignees: Vec<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub add_projects: Vec<String>,
    pub remove_projects: Vec<String>,
    pub milestone: Option<String>,
}

impl IssueEdit {
    pub fn new(issue: impl Into<String>) -> Self {
        Self {
            issue: issue.into(),
            repo: None,
            title: None,
            body: None,
            add_assignees: Vec::new(),
            remove_assignees: Vec::new(),
            add_labels: Vec::new(),
            remove_labels: Vec::new(),
            add_projects: Vec::new(),
            remove_projects: Vec::new(),
            milestone: None,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["issue".into(), "edit".into(), self.issue.clone().into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--title", &self.title);
        push_opt(&mut args, "--body", &self.body);
        push_repeated(&mut args, "--add-assignee", &self.add_assignees);
        push_repeated(&mut args, "--remove-assignee", &self.remove_assignees);
        push_repeated(&mut args, "--add-label", &self.add_labels);
        push_repeated(&mut args, "--remove-label", &self.remove_labels);
        push_repeated(&mut args, "--add-project", &self.add_projects);
        push_repeated(&mut args, "--remove-project", &self.remove_projects);
        push_opt(&mut args, "--milestone", &self.milestone);
        args
    }
}

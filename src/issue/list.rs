use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::{push_opt, push_opt_u32, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Issue state accepted by `gh issue list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueState {
    Open,
    Closed,
    All,
}

impl IssueState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::All => "all",
        }
    }
}

/// Options for `gh issue list`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IssueList {
    pub repo: Option<String>,
    pub state: Option<IssueState>,
    pub assignee: Option<String>,
    pub author: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
    pub json_fields: Vec<String>,
}

impl IssueList {
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
        let mut args = vec!["issue".into(), "list".into()];
        push_opt(&mut args, "--repo", &self.repo);
        if let Some(state) = self.state {
            args.push("--state".into());
            args.push(state.as_str().into());
        }
        push_opt(&mut args, "--assignee", &self.assignee);
        push_opt(&mut args, "--author", &self.author);
        push_repeated(&mut args, "--label", &self.labels);
        push_opt(&mut args, "--milestone", &self.milestone);
        push_opt(&mut args, "--search", &self.search);
        push_opt_u32(&mut args, "--limit", self.limit);
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        args
    }
}

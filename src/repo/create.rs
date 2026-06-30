use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Repository visibility accepted by `gh repo create`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoVisibility {
    Public,
    Private,
    Internal,
}

impl RepoVisibility {
    fn flag(self) -> &'static str {
        match self {
            Self::Public => "--public",
            Self::Private => "--private",
            Self::Internal => "--internal",
        }
    }
}

/// Options for `gh repo create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoCreate {
    pub name: String,
    pub visibility: Option<RepoVisibility>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub source: Option<String>,
    pub remote: Option<String>,
    pub team: Option<String>,
    pub clone: bool,
    pub disable_issues: bool,
    pub disable_wiki: bool,
    pub add_readme: bool,
}

impl RepoCreate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visibility: None,
            description: None,
            homepage: None,
            source: None,
            remote: None,
            team: None,
            clone: false,
            disable_issues: false,
            disable_wiki: false,
            add_readme: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["repo".into(), "create".into(), self.name.clone().into()];
        if let Some(visibility) = self.visibility {
            args.push(visibility.flag().into());
        }
        push_opt(&mut args, "--description", &self.description);
        push_opt(&mut args, "--homepage", &self.homepage);
        push_opt(&mut args, "--source", &self.source);
        push_opt(&mut args, "--remote", &self.remote);
        push_opt(&mut args, "--team", &self.team);
        push_bool(&mut args, "--clone", self.clone);
        push_bool(&mut args, "--disable-issues", self.disable_issues);
        push_bool(&mut args, "--disable-wiki", self.disable_wiki);
        push_bool(&mut args, "--add-readme", self.add_readme);
        args
    }
}

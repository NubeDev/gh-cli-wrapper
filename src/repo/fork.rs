use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh repo fork`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFork {
    pub repository: Option<String>,
    pub org: Option<String>,
    pub remote: bool,
    pub clone: bool,
}

impl RepoFork {
    pub fn new(repository: Option<impl Into<String>>) -> Self {
        Self {
            repository: repository.map(Into::into),
            org: None,
            remote: false,
            clone: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["repo".into(), "fork".into()];
        if let Some(repository) = &self.repository {
            args.push(repository.into());
        }
        push_opt(&mut args, "--org", &self.org);
        push_bool(&mut args, "--remote", self.remote);
        push_bool(&mut args, "--clone", self.clone);
        args
    }
}

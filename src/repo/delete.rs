use std::ffi::OsString;

use crate::args::push_bool;
use crate::{Error, Gh, GhOutput};

/// Options for `gh repo delete`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoDelete {
    pub repository: Option<String>,
    pub yes: bool,
}

impl RepoDelete {
    pub fn new(repository: Option<impl Into<String>>) -> Self {
        Self {
            repository: repository.map(Into::into),
            yes: true,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["repo".into(), "delete".into()];
        if let Some(repository) = &self.repository {
            args.push(repository.into());
        }
        push_bool(&mut args, "--yes", self.yes);
        args
    }
}

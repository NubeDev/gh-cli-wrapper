use std::ffi::OsString;

use crate::{Error, Gh, GhOutput};

/// Options for `gh repo clone`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoClone {
    pub repository: String,
    pub directory: Option<String>,
    pub git_args: Vec<String>,
}

impl RepoClone {
    pub fn new(repository: impl Into<String>) -> Self {
        Self {
            repository: repository.into(),
            directory: None,
            git_args: Vec::new(),
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec![
            "repo".into(),
            "clone".into(),
            self.repository.clone().into(),
        ];
        if let Some(directory) = &self.directory {
            args.push(directory.into());
        }
        if !self.git_args.is_empty() {
            args.push("--".into());
            args.extend(self.git_args.iter().map(Into::into));
        }
        args
    }
}

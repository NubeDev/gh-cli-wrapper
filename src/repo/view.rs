use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::push_bool;
use crate::{Error, Gh, GhOutput};

/// Options for `gh repo view`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoView {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub json_fields: Vec<String>,
    pub web: bool,
}

impl RepoView {
    pub fn new(repository: Option<impl Into<String>>) -> Self {
        Self {
            repository: repository.map(Into::into),
            branch: None,
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
        let mut args = vec!["repo".into(), "view".into()];
        if let Some(repository) = &self.repository {
            args.push(repository.into());
        }
        if let Some(branch) = &self.branch {
            args.push("--branch".into());
            args.push(branch.into());
        }
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        push_bool(&mut args, "--web", self.web);
        args
    }
}

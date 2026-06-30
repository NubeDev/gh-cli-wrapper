use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::{push_bool, push_opt, push_opt_u32};
use crate::{Error, Gh, GhOutput};

/// Options for `gh repo list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoList {
    pub owner: Option<String>,
    pub archived: bool,
    pub fork: bool,
    pub language: Option<String>,
    pub limit: Option<u32>,
    pub source: bool,
    pub topic: Option<String>,
    pub visibility: Option<String>,
    pub json_fields: Vec<String>,
}

impl Default for RepoList {
    fn default() -> Self {
        Self {
            owner: None,
            archived: false,
            fork: false,
            language: None,
            limit: Some(30),
            source: false,
            topic: None,
            visibility: None,
            json_fields: Vec::new(),
        }
    }
}

impl RepoList {
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
        let mut args = vec!["repo".into(), "list".into()];
        if let Some(owner) = &self.owner {
            args.push(owner.into());
        }
        push_bool(&mut args, "--archived", self.archived);
        push_bool(&mut args, "--fork", self.fork);
        push_opt(&mut args, "--language", &self.language);
        push_opt_u32(&mut args, "--limit", self.limit);
        push_bool(&mut args, "--source", self.source);
        push_opt(&mut args, "--topic", &self.topic);
        push_opt(&mut args, "--visibility", &self.visibility);
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        args
    }
}

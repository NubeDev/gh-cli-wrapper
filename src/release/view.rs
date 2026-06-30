use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh release view`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseView {
    pub tag: Option<String>,
    pub repo: Option<String>,
    pub json_fields: Vec<String>,
    pub web: bool,
}

impl ReleaseView {
    pub fn new(tag: Option<impl Into<String>>) -> Self {
        Self {
            tag: tag.map(Into::into),
            repo: None,
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
        let mut args = vec!["release".into(), "view".into()];
        if let Some(tag) = &self.tag {
            args.push(tag.into());
        }
        push_opt(&mut args, "--repo", &self.repo);
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        push_bool(&mut args, "--web", self.web);
        args
    }
}

use std::ffi::OsString;

use serde::de::DeserializeOwned;

use crate::args::{push_opt, push_opt_u32};
use crate::{Error, Gh, GhOutput};

/// Options for `gh release list`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReleaseList {
    pub repo: Option<String>,
    pub limit: Option<u32>,
    pub json_fields: Vec<String>,
}

impl ReleaseList {
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
        let mut args = vec!["release".into(), "list".into()];
        push_opt(&mut args, "--repo", &self.repo);
        push_opt_u32(&mut args, "--limit", self.limit);
        if !self.json_fields.is_empty() {
            args.push("--json".into());
            args.push(self.json_fields.join(",").into());
        }
        args
    }
}

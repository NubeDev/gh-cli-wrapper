use std::ffi::OsString;

use crate::args::{push_bool, push_opt};
use crate::{Error, Gh, GhOutput};

/// Options for `gh release create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCreate {
    pub tag: String,
    pub repo: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub target: Option<String>,
    pub assets: Vec<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub generate_notes: bool,
}

impl ReleaseCreate {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            repo: None,
            title: None,
            notes: None,
            target: None,
            assets: Vec::new(),
            draft: false,
            prerelease: false,
            generate_notes: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["release".into(), "create".into(), self.tag.clone().into()];
        args.extend(self.assets.iter().map(Into::into));
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--title", &self.title);
        push_opt(&mut args, "--notes", &self.notes);
        push_opt(&mut args, "--target", &self.target);
        push_bool(&mut args, "--draft", self.draft);
        push_bool(&mut args, "--prerelease", self.prerelease);
        push_bool(&mut args, "--generate-notes", self.generate_notes);
        args
    }
}

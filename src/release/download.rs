use std::ffi::OsString;

use crate::args::{push_bool, push_opt, push_repeated};
use crate::{Error, Gh, GhOutput};

/// Options for `gh release download`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDownload {
    pub tag: Option<String>,
    pub repo: Option<String>,
    pub archive: Option<String>,
    pub directory: Option<String>,
    pub patterns: Vec<String>,
    pub clobber: bool,
}

impl ReleaseDownload {
    pub fn new(tag: Option<impl Into<String>>) -> Self {
        Self {
            tag: tag.map(Into::into),
            repo: None,
            archive: None,
            directory: None,
            patterns: Vec::new(),
            clobber: false,
        }
    }

    pub fn run(&self, gh: &Gh) -> Result<GhOutput, Error> {
        gh.args(self.args())
    }

    pub fn args(&self) -> Vec<OsString> {
        let mut args = vec!["release".into(), "download".into()];
        if let Some(tag) = &self.tag {
            args.push(tag.into());
        }
        push_opt(&mut args, "--repo", &self.repo);
        push_opt(&mut args, "--archive", &self.archive);
        push_opt(&mut args, "--dir", &self.directory);
        push_repeated(&mut args, "--pattern", &self.patterns);
        push_bool(&mut args, "--clobber", self.clobber);
        args
    }
}

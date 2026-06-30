use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git push`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitPush {
    pub cwd: Option<PathBuf>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub set_upstream: bool,
    pub force_with_lease: bool,
    pub tags: bool,
    pub extra_args: Vec<String>,
}

impl GitPush {
    pub fn run(&self) -> Result<Output, Error> {
        let mut command = Command::new("git");
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command.args(self.args()).stdin(Stdio::null());
        let output = command.output()?;
        Error::from_output("git", output)
    }

    pub fn args(&self) -> Vec<String> {
        let mut args = vec!["push".to_string()];
        if self.set_upstream {
            args.push("--set-upstream".to_string());
        }
        if self.force_with_lease {
            args.push("--force-with-lease".to_string());
        }
        if self.tags {
            args.push("--tags".to_string());
        }
        if let Some(remote) = &self.remote {
            args.push(remote.clone());
        }
        if let Some(branch) = &self.branch {
            args.push(branch.clone());
        }
        args.extend(self.extra_args.clone());
        args
    }
}

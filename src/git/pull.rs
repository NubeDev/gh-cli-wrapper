use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git pull`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitPull {
    pub cwd: Option<PathBuf>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub rebase: bool,
    pub ff_only: bool,
    pub extra_args: Vec<String>,
}

impl GitPull {
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
        let mut args = vec!["pull".to_string()];
        if self.rebase {
            args.push("--rebase".to_string());
        }
        if self.ff_only {
            args.push("--ff-only".to_string());
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

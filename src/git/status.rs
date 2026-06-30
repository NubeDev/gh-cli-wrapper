use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git status`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitStatus {
    pub cwd: Option<PathBuf>,
    pub short: bool,
    pub branch: bool,
    pub porcelain: Option<String>,
}

impl GitStatus {
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
        let mut args = vec!["status".to_string()];
        if self.short {
            args.push("--short".to_string());
        }
        if self.branch {
            args.push("--branch".to_string());
        }
        if let Some(porcelain) = &self.porcelain {
            args.push(format!("--porcelain={porcelain}"));
        }
        args
    }
}

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git checkout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCheckout {
    pub branch: String,
    pub cwd: Option<PathBuf>,
    pub create: bool,
    pub force: bool,
}

impl GitCheckout {
    pub fn new(branch: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
            cwd: None,
            create: false,
            force: false,
        }
    }

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
        let mut args = vec!["checkout".to_string()];
        if self.create {
            args.push("-b".to_string());
        }
        if self.force {
            args.push("--force".to_string());
        }
        args.push(self.branch.clone());
        args
    }
}

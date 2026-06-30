use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git add`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitAdd {
    pub cwd: Option<PathBuf>,
    pub paths: Vec<String>,
    pub all: bool,
}

impl GitAdd {
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
        let mut args = vec!["add".to_string()];
        if self.all {
            args.push("--all".to_string());
        }
        args.extend(self.paths.clone());
        args
    }
}

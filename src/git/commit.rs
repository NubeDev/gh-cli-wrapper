use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::Error;

/// Options for `git commit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommit {
    pub message: String,
    pub cwd: Option<PathBuf>,
    pub all: bool,
    pub amend: bool,
    pub no_edit: bool,
    pub signoff: bool,
    pub extra_args: Vec<String>,
}

impl GitCommit {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cwd: None,
            all: false,
            amend: false,
            no_edit: false,
            signoff: false,
            extra_args: Vec::new(),
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
        let mut args = vec![
            "commit".to_string(),
            "--message".to_string(),
            self.message.clone(),
        ];
        if self.all {
            args.push("--all".to_string());
        }
        if self.amend {
            args.push("--amend".to_string());
        }
        if self.no_edit {
            args.push("--no-edit".to_string());
        }
        if self.signoff {
            args.push("--signoff".to_string());
        }
        args.extend(self.extra_args.clone());
        args
    }
}

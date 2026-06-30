use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;

use crate::Error;

/// Thin wrapper around the GitHub CLI executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gh {
    executable: PathBuf,
}

impl Gh {
    /// Use `gh` from `PATH`.
    pub fn new() -> Self {
        Self {
            executable: PathBuf::from("gh"),
        }
    }

    /// Use a specific `gh` executable path, useful for tests or bundled tools.
    pub fn with_executable(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    /// Return whether the executable can be started.
    pub fn is_available(&self) -> bool {
        self.args(["--version"]).is_ok()
    }

    /// Run `gh` with raw args and return captured output.
    pub fn args<I, S>(&self, args: I) -> Result<GhOutput, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run(args.into_iter().map(|arg| arg.as_ref().to_os_string()))
    }

    /// Run `gh api <endpoint>` and decode the JSON response.
    pub fn api_json<T>(&self, hostname: Option<&str>, endpoint: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut args = vec![OsString::from("api")];
        if let Some(hostname) = hostname {
            args.push(OsString::from("--hostname"));
            args.push(OsString::from(hostname));
        }
        args.push(OsString::from(endpoint));
        self.json(args)
    }

    /// Run any JSON-producing `gh` command and decode stdout.
    pub fn json<I, S, T>(&self, args: I) -> Result<T, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        T: DeserializeOwned,
    {
        let output = self.args(args)?;
        Ok(serde_json::from_slice(&output.stdout)?)
    }

    fn run<I>(&self, args: I) -> Result<GhOutput, Error>
    where
        I: IntoIterator<Item = OsString>,
    {
        let output = Command::new(&self.executable)
            .args(args)
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            return Err(Error::GhStatus {
                status: output.status,
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(GhOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

impl Default for Gh {
    fn default() -> Self {
        Self::new()
    }
}

/// Captured `gh` process output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

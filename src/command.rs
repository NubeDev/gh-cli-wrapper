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

    /// A logged command for this executable — the one constructor, so a typed builder's
    /// call lands in the call log like any other.
    #[track_caller]
    pub(crate) fn command(&self) -> Command {
        crate::call::command_for(&self.executable)
    }

    /// Return whether the executable can be started.
    #[track_caller]
    pub fn is_available(&self) -> bool {
        self.args(["--version"]).is_ok()
    }

    /// Run `gh` with raw args and return captured output.
    ///
    /// A failure is classified the way [`crate::run`] classifies it: `gh` missing, rate
    /// limited, offline, or refused are different [`Error`] variants.
    #[track_caller]
    pub fn args<I, S>(&self, args: I) -> Result<GhOutput, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run(args.into_iter().map(|arg| arg.as_ref().to_os_string()))
    }

    /// Run `gh api <endpoint>` and decode the JSON response.
    #[track_caller]
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
    #[track_caller]
    pub fn json<I, S, T>(&self, args: I) -> Result<T, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        T: DeserializeOwned,
    {
        let output = self.args(args)?;
        Ok(serde_json::from_slice(&output.stdout)?)
    }

    #[track_caller]
    fn run<I>(&self, args: I) -> Result<GhOutput, Error>
    where
        I: IntoIterator<Item = OsString>,
    {
        let args: Vec<OsString> = args.into_iter().collect();
        let output = crate::call::output(self.command().args(&args).stdin(Stdio::null()))?;
        let lossy: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let argv: Vec<&str> = lossy.iter().map(String::as_str).collect();
        crate::call::finished(&argv, &output);

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if let Some(limited) = crate::rate_limit::parse(&format!("{stderr}\n{stdout}")) {
                return Err(limited);
            }
            if crate::rate_limit::is_network_error(&stderr) {
                return Err(Error::Offline(stderr.trim().to_string()));
            }
            return Err(Error::Failed {
                command_line: crate::run::rendered(&argv),
                code: output.status.code().unwrap_or(-1),
                stderr: stderr.trim().to_string(),
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

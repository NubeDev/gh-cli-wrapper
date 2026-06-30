use std::io;
use std::process::ExitStatus;

use thiserror::Error as ThisError;

/// Errors returned by the GitHub CLI wrapper.
#[derive(Debug, ThisError)]
pub enum Error {
    #[error("GitHub CLI config error: {0}")]
    Config(#[from] gh_config::Error),

    #[error("failed to start gh: {0}")]
    Io(#[from] io::Error),

    #[error("gh exited with {status}: {stderr}")]
    GhStatus { status: ExitStatus, stderr: String },

    #[error("{program} exited with {status}: {stderr}")]
    CommandStatus {
        program: String,
        status: ExitStatus,
        stderr: String,
    },

    #[error("failed to decode gh JSON output: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to decode GitHub CLI host config: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl Error {
    pub(crate) fn from_output(
        program: impl Into<String>,
        output: std::process::Output,
    ) -> Result<std::process::Output, Self> {
        if output.status.success() {
            return Ok(output);
        }

        Err(Self::CommandStatus {
            program: program.into(),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

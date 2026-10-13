use std::io;

use thiserror::Error as ThisError;

use crate::auth::MissingScope;

/// Errors returned by the GitHub CLI wrapper.
///
/// **The failures with different remedies are different variants.** `gh` missing is fixed by
/// installing it, a rate limit by waiting for the reset, no network by trying again, and a
/// refused call by reading `gh`'s own words. A caller that wants to say the right thing to a
/// person switches on the variant; it never parses the message back out.
#[derive(Debug, ThisError)]
pub enum Error {
    /// The `gh` executable could not be started. `gh auth login` cannot fix this.
    #[error("`gh` is not on PATH — install it from cli.github.com")]
    GhMissing,

    /// `gh` ran and exited non-zero. `stderr` is `gh`'s own words, verbatim; `command_line`
    /// is what was run, so the failure can be reproduced by pasting it.
    #[error("`{command_line}` exited {code}\n{stderr}")]
    Failed {
        command_line: String,
        code: i32,
        stderr: String,
    },

    /// GitHub refused the call for the rate limit. Never retry into it: stop and report the
    /// reset. `limit` of 0 means the refusal carried no budget (a GraphQL refusal has no
    /// headers); `reset` is Unix seconds, or `"unknown"` when nobody said.
    #[error("rate limited — {remaining} of {limit} remaining, resets at {reset}")]
    RateLimited {
        remaining: u32,
        limit: u32,
        reset: String,
    },

    /// The network did not answer. Distinct from a rate limit: one retry, then stop.
    #[error("cannot reach GitHub: {0}")]
    Offline(String),

    /// Logged in, `gh` present, and the token lacks a scope the call needed. The remedy is
    /// `gh auth refresh`, never `gh auth login` — see [`crate::auth::scope`].
    #[error("the token is missing the {} scope(s). Run: {}", .0.scopes.join(", "), .0.remedy)]
    MissingScope(MissingScope),

    /// A program other than `gh` (`git`) exited non-zero.
    #[error("{program} exited with {status}: {stderr}")]
    CommandStatus {
        program: String,
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[error("GitHub CLI config error: {0}")]
    Config(#[from] gh_config::Error),

    #[error("failed to start gh: {0}")]
    Io(#[from] io::Error),

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

    /// A spawn failure: `NotFound` is [`Error::GhMissing`], anything else is I/O.
    pub(crate) fn spawn(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::NotFound {
            Self::GhMissing
        } else {
            Self::Io(err)
        }
    }
}

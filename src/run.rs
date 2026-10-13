//! **Run one `gh` invocation and classify how it failed** — the raw runner every verb the
//! typed builders do not cover goes through.
//!
//! "`gh` is missing", "you are rate limited", "you are offline" and "that call was refused"
//! have four different remedies, so they are four different [`Error`] variants — decided
//! here, once, so no caller can collapse two of them that another keeps apart.

use serde::de::DeserializeOwned;

use crate::Error;

/// Run `gh <args>` and return stdout, or the failure as the variant its remedy needs.
#[track_caller]
pub fn stdout(args: &[&str]) -> Result<Vec<u8>, Error> {
    stdout_as(args, rendered(args))
}

/// [`stdout`], with the command line a failure reports chosen by the caller — the one a
/// confirm step already showed, say.
#[track_caller]
pub fn stdout_as(args: &[&str], command_line: String) -> Result<Vec<u8>, Error> {
    let output = crate::call::output(crate::call::command().args(args))?;
    crate::call::finished(args, &output);

    if output.status.success() {
        return Ok(output.stdout);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // `gh api` puts GitHub's message on stderr and the body (headers too, with `-i`) on
    // stdout, so both are searched.
    if let Some(limited) = crate::rate_limit::parse(&format!("{stderr}\n{stdout}")) {
        return Err(limited);
    }
    if crate::rate_limit::is_network_error(&stderr) {
        return Err(Error::Offline(stderr.trim().to_string()));
    }
    Err(Error::Failed {
        command_line,
        code: output.status.code().unwrap_or(-1),
        stderr: stderr.trim().to_string(),
    })
}

/// [`stdout`], as text — what most callers want from a `gh` subcommand.
#[track_caller]
pub fn text(args: &[&str]) -> Result<String, Error> {
    text_as(args, rendered(args))
}

/// [`stdout_as`], as text.
#[track_caller]
pub fn text_as(args: &[&str], command_line: String) -> Result<String, Error> {
    let stdout = stdout_as(args, command_line)?;
    Ok(String::from_utf8_lossy(&stdout).into_owned())
}

/// [`stdout`] for its exit status alone — a write that confirms on stderr and prints nothing
/// worth parsing.
#[track_caller]
pub fn status(args: &[&str]) -> Result<(), Error> {
    stdout(args).map(drop)
}

/// [`stdout`], parsed as JSON. A body that does not parse is a [`Error::Failed`] naming the
/// command, not a bare decode error with no context.
#[track_caller]
pub fn json<T: DeserializeOwned>(args: &[&str]) -> Result<T, Error> {
    let stdout = stdout(args)?;
    serde_json::from_slice(&stdout).map_err(|e| Error::Failed {
        command_line: rendered(args),
        code: 0,
        stderr: format!("could not parse gh's JSON output: {e}"),
    })
}

/// `gh api <path> --paginate`, parsed.
///
/// **`--paginate` is not optional for a list.** `gh api` returns one page, and a truncated
/// list is a wrong answer that looks like a right one. There is no `--slurp` on `gh` 2.45;
/// `--paginate` already merges JSON *arrays* into one array — which holds for the
/// array-returning endpoints this is for, and not for an object-returning one.
#[track_caller]
pub fn api_paginated<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    json(&["api", path, "--paginate"])
}

/// The same runner, but **a non-zero exit is not a failure** — the response is.
///
/// `gh api -i` prints the status line, the headers and the body on stdout for a 404 or a
/// 403 and *then* exits 1. A 404 is an answer to classify, not an error to raise. The three
/// conditions that really are failures still are: no `gh`, no network, and a rate limit —
/// facts about the *machine*, none of which produces a status line to read.
#[track_caller]
pub fn allowing_failure(args: &[&str]) -> Result<Vec<u8>, Error> {
    let output = crate::call::output(crate::call::command().args(args))?;
    crate::call::finished(args, &output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(limited) = crate::rate_limit::parse(&stdout) {
        return Err(limited);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if crate::rate_limit::is_network_error(&stderr) {
        return Err(Error::Offline(stderr.trim().to_string()));
    }
    Ok(output.stdout)
}

/// The command line `args` runs, quoted so it can be pasted into a shell and run.
///
/// A write that shows its command before it runs should show **this** string, from the same
/// argv it executes — so the confirm cannot drift from what runs.
pub fn rendered(args: &[&str]) -> String {
    let mut line = String::from("gh");
    for arg in args {
        line.push(' ');
        if arg.is_empty() || arg.contains(|c: char| c.is_whitespace() || c == '"') {
            line.push('"');
            line.push_str(&arg.replace('\\', "\\\\").replace('"', "\\\""));
            line.push('"');
        } else {
            line.push_str(arg);
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rendered_line_quotes_what_a_shell_would_need() {
        assert_eq!(
            rendered(&["issue", "create", "--title", "Fix \"it\" now", "--body", ""]),
            r#"gh issue create --title "Fix \"it\" now" --body """#
        );
    }
}

//! **Run one `gh` invocation and classify how it failed** — the raw runner every verb the
//! typed builders do not cover goes through.
//!
//! "`gh` is missing", "you are rate limited", "you are offline" and "that call was refused"
//! have four different remedies, so they are four different [`Error`] variants — decided
//! here, once, so no caller can collapse two of them that another keeps apart.

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

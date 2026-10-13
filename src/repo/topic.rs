//! `gh repo edit --add-topic` / `--remove-topic` — one topic on one repository.
//!
//! Verified against the real CLI (`gh repo edit --help`, gh 2.45.0):
//! `gh repo edit [<repository>] --add-topic strings` / `--remove-topic strings`.
//!
//! **This is a write.** GitHub caps topics per repository.

use crate::Error;

#[track_caller]
pub fn add_topic(repo: &str, topic: &str) -> Result<(), Error> {
    edit(repo, "--add-topic", topic)
}

#[track_caller]
pub fn remove_topic(repo: &str, topic: &str) -> Result<(), Error> {
    edit(repo, "--remove-topic", topic)
}

#[track_caller]
fn edit(repo: &str, flag: &str, topic: &str) -> Result<(), Error> {
    let args = ["repo", "edit", repo, flag, topic];
    let output = crate::call::output(crate::call::command().args(args))?;
    crate::call::finished(&args, &output);

    if output.status.success() {
        return Ok(());
    }
    // Surfaced verbatim, prefixed with the command line that ran — never swallowed.
    Err(Error::Failed {
        command_line: format!("gh repo edit {repo} {flag} {topic}"),
        code: output.status.code().unwrap_or(-1),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

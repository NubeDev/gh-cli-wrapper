//! The scopes the stored token holds, **as `gh` reads them now**.
//!
//! A `gh auth refresh` is not finished when it exits 0; it is finished when the token can do
//! what was asked — which is this read. `gh auth status --json hosts --hostname <h>` always
//! exits 0 (gh 2.94.0) and reports each account's `scopes` as one comma-separated string.
//!
//! **`--json` is newer than the flag-less form.** gh 2.45.0 (Ubuntu's apt package) answers
//! `unknown flag: --json` on stdout-empty, and reading that as JSON failed every grant with
//! `EOF while parsing` — *after* the token had the scope. On such a `gh` we read the text
//! form's `Token scopes:` line instead, the only answer it gives (`credential.rs` falls back
//! the same way, for the same flag).

use crate::Error;

/// The active account's scopes on `host`, or `None` when `gh` reports no account there.
/// Always asks `gh` — `gh auth status` checks the token over the network, so a caller that
/// asks on every request should remember the answer.
#[track_caller]
pub fn held_scopes(host: &str) -> Result<Option<Vec<String>>, Error> {
    let output = crate::call::output(crate::call::command().args([
        "auth",
        "status",
        "--json",
        "hosts",
        "--hostname",
        host,
    ]))?;
    crate::call::finished(&["auth", "status"], &output);
    match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        Ok(body) => Ok(parse(&body, host)),
        Err(_) => held_scopes_text(host),
    }
}

/// The fallback for a `gh` without `auth status --json`: the text form, on either stream
/// (older `gh` printed status to stderr, 2.45.0 prints it to stdout).
#[track_caller]
fn held_scopes_text(host: &str) -> Result<Option<Vec<String>>, Error> {
    let output =
        crate::call::output(crate::call::command().args(["auth", "status", "--hostname", host]))?;
    crate::call::finished(&["auth", "status"], &output);
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(parse_text(&text))
}

/// The active account's scopes, from `gh auth status`'s text. Each account is a block
/// opening at `Logged in to`; the one with `Active account: true` wins, else the first.
pub fn parse_text(text: &str) -> Option<Vec<String>> {
    let mut accounts: Vec<(bool, Vec<String>)> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.contains("Logged in to") {
            accounts.push((false, Vec::new()));
        } else if let Some(account) = accounts.last_mut() {
            if line.ends_with("Active account: true") {
                account.0 = true;
            } else if let Some((_, list)) = line.split_once("Token scopes:") {
                account.1 = list
                    .split(',')
                    .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                    .filter(|s| !s.is_empty() && s != "none")
                    .collect();
            }
        }
    }
    let active = accounts.iter().position(|a| a.0).unwrap_or(0);
    accounts.into_iter().nth(active).map(|a| a.1)
}

/// The active account's scopes, from `gh auth status --json hosts`.
pub fn parse(body: &serde_json::Value, host: &str) -> Option<Vec<String>> {
    let accounts = body["hosts"][host].as_array()?;
    let active = accounts
        .iter()
        .find(|a| a["active"] == true)
        .or(accounts.first())?;
    Some(
        active["scopes"]
            .as_str()
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real body, token source and all, with nothing secret in it.
    #[test]
    fn the_active_accounts_scopes_are_read() {
        let body = serde_json::json!({ "hosts": { "github.com": [
            { "state": "success", "active": false, "login": "other", "scopes": "repo" },
            { "state": "success", "active": true, "host": "github.com", "login": "someone",
              "tokenSource": "keyring", "scopes": "gist, project, read:org, repo, workflow",
              "gitProtocol": "https" }
        ] } });
        assert_eq!(
            parse(&body, "github.com").unwrap(),
            ["gist", "project", "read:org", "repo", "workflow"]
        );
    }

    /// gh 2.45.0's text, verbatim but for the token.
    #[test]
    fn the_text_forms_scopes_are_read() {
        let text = "github.com\n  \u{2713} Logged in to github.com account someone (keyring)\n  \
                    - Active account: true\n  - Git operations protocol: https\n  \
                    - Token: gho_****\n  - Token scopes: 'gist', 'project', 'read:org', 'repo', 'workflow'\n";
        assert_eq!(
            parse_text(text).unwrap(),
            ["gist", "project", "read:org", "repo", "workflow"]
        );
    }

    /// Logged out: no account block, so no scopes to report.
    #[test]
    fn the_text_form_logged_out_is_none() {
        assert_eq!(
            parse_text("You are not logged into any GitHub hosts."),
            None
        );
    }

    /// `gh` answers `{"hosts":{}}` for a host it has no account on — not an error.
    #[test]
    fn no_account_on_the_host_is_none() {
        assert_eq!(
            parse(&serde_json::json!({ "hosts": {} }), "nope.example"),
            None
        );
    }
}

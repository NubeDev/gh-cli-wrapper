//! What `gh` can say about the stored credential for a host — **without deciding what
//! that means for the caller**.
//!
//! # Why not the exit code of `gh auth status`
//!
//! On `gh` 2.45.0 a logged-out `gh` printed *"You are not logged into any GitHub hosts"* and
//! the check was local. **`gh` 2.94.0 validates the token over the network**, so on a machine
//! with a perfectly good credential and no network, `gh auth status` exits 1 — and reading
//! that as "logged out" sends someone to `gh auth login`, which cannot fix a closed port.
//!
//! # Why the JSON form
//!
//! `gh auth status --json hosts` **always exits 0** (`gh auth status --help`, 2.94.0) and
//! reports per-account `state` plus, on failure, `gh`'s own `error` string. So the
//! classification is a reading of what `gh` said rather than an inference from an exit code.
//!
//! # Distinguishing rejected from unverifiable
//!
//! The discriminator is **whether GitHub answered at all**, because a rejection is a thing
//! only the server can do: `gh` reports `non-200 OK status code: 401 Unauthorized` when the
//! credential reached GitHub and came back refused, and a transport error (`dial tcp`, `no
//! such host`, `i/o timeout`, `proxyconnect`) when it did not. We match on the presence of a
//! status code rather than on a list of network phrases, because the list of ways a network
//! fails is open and the set of ways a status code arrives is not.
//!
//! Anything else — a 500, a captive portal's 200 of HTML — is [`Credential::Unverified`], not
//! a guess. `gh`'s own reason is carried verbatim.

use crate::{Error, Gh};

/// What `gh` knows about the credential for one host.
///
/// Five conditions, because they have four different remedies. [`Absent`] and
/// [`Rejected`] share the remedy `gh auth login` and so are allowed to collapse into
/// [`super::AuthState::Unauthenticated`]; [`Unverified`] is the one that must not.
///
/// [`Absent`]: Credential::Absent
/// [`Rejected`]: Credential::Rejected
/// [`Unverified`]: Credential::Unverified
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// `gh` validated the stored credential against the host.
    Valid,
    /// No credential is stored for the host. A **local** fact: `gh` answers it without a
    /// network, and answers it the same way on a plane.
    Absent,
    /// A credential is stored and GitHub refused it. Carries `gh`'s own reason.
    Rejected(String),
    /// A credential is stored and `gh` could not find out whether it works — the host did
    /// not answer, or answered something that is not a verdict. Carries `gh`'s own
    /// reason. **Never worded as being logged out.**
    Unverified(String),
    /// `gh` is not on `PATH`. `gh auth login` cannot fix it.
    GhMissing,
}

/// Ask `gh` about the credential for `host` (its default host when `None`).
///
/// We never read, store or expose a token: `gh` holds the credential and we inherit its
/// auth, refresh, SSO and enterprise hosts by delegating.
///
#[track_caller]
pub fn credential(host: Option<&str>) -> Result<Credential, Error> {
    credential_with(&Gh::new(), host)
}

/// [`credential`] through a caller-chosen `gh`.
#[track_caller]
pub fn credential_with(gh: &Gh, host: Option<&str>) -> Result<Credential, Error> {
    let mut command = gh.command();
    command.args(["auth", "status", "--json", "hosts"]);
    if let Some(host) = host {
        command.args(["--hostname", host]);
    }

    let output = match command.output() {
        Ok(output) => output,
        // A different remedy from being logged out.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Credential::GhMissing),
        Err(err) => return Err(err.into()),
    };
    crate::call::finished(&["auth", "status"], &output);

    match serde_json::from_slice::<Report>(&output.stdout) {
        Ok(report) => Ok(classify(&report, host)),
        // `--json` on `gh auth status` is newer than the flagless form. On a `gh` that
        // does not have it there is no structured answer to read, so we fall back to the
        // exit-code reading — no worse, and the only reading available.
        Err(_) => Ok(fallback(gh, host)),
    }
}

/// `gh auth status --json hosts`: host -> the accounts known for it.
#[derive(serde::Deserialize)]
struct Report {
    hosts: std::collections::HashMap<String, Vec<Account>>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Account {
    /// `"success"` or `"error"`.
    state: String,
    #[serde(default)]
    active: bool,
    /// Present when `state` is `"error"` — `gh`'s own words for what went wrong.
    #[serde(default)]
    error: String,
}

fn classify(report: &Report, host: Option<&str>) -> Credential {
    // With `--hostname` the map holds that host alone; without it, the default host is
    // the one whose accounts are marked active.
    let accounts = match host.and_then(|h| report.hosts.get(h)) {
        Some(accounts) => accounts,
        None if host.is_some() => return Credential::Absent,
        None => match report.hosts.values().find(|a| a.iter().any(|a| a.active)) {
            Some(accounts) => accounts,
            // `{"hosts":{}}` — logged out of everything, which `gh` decides locally.
            None => return Credential::Absent,
        },
    };

    // The active account is the one a later `gh` call will use; without one, the first
    // is what `gh` itself falls back to.
    let account = match accounts.iter().find(|a| a.active).or(accounts.first()) {
        Some(account) => account,
        None => return Credential::Absent,
    };

    if account.state == "success" {
        return Credential::Valid;
    }
    if answered_with_a_status(&account.error) {
        Credential::Rejected(account.error.clone())
    } else {
        Credential::Unverified(account.error.clone())
    }
}

/// Did GitHub answer? A status code can only exist if it did — see the module note on why
/// this is asked in the positive rather than by listing the ways a network fails.
///
/// Only 401 and 403 are a verdict on the credential. Every other status means the host
/// answered but said nothing about whether the token is good, which is
/// [`Credential::Unverified`], not a guess dressed as a login prompt.
fn answered_with_a_status(error: &str) -> bool {
    let Some(rest) = error.split("status code:").nth(1) else {
        return false;
    };
    let code = rest.split_whitespace().next().unwrap_or_default();
    matches!(code, "401" | "403")
}

/// The pre-`--json` reading: exit zero is authenticated, anything else is not. Kept only
/// for a `gh` too old to answer in JSON; it cannot tell offline from logged out, which is
/// the whole reason the JSON form is preferred above it.
#[track_caller]
fn fallback(gh: &Gh, host: Option<&str>) -> Credential {
    let mut command = gh.command();
    command.args(["auth", "status"]);
    if let Some(host) = host {
        command.args(["--hostname", host]);
    }
    let output = command.output();
    if let Ok(output) = &output {
        crate::call::finished(&["auth", "status"], output);
    }
    match output {
        Ok(output) if output.status.success() => Credential::Valid,
        Ok(_) => Credential::Absent,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Credential::GhMissing,
        Err(_) => Credential::Absent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real bodies, captured from `gh 2.94.0` on 2026-09-09 and pasted verbatim — a
    /// committed response body for a parser test, so no code path believes it reached
    /// GitHub.
    fn parse(body: &str, host: Option<&str>) -> Credential {
        classify(&serde_json::from_str::<Report>(body).unwrap(), host)
    }

    const SUCCESS: &str = r#"{"hosts":{"github.com":[{"state":"success","active":true,
        "host":"github.com","login":"someone","tokenSource":"keyring",
        "scopes":"gist, read:org, repo, workflow","gitProtocol":"https"}]}}"#;

    /// Captured with `HTTPS_PROXY` at a closed loopback port — a real offline `gh`.
    const OFFLINE: &str = r#"{"hosts":{"github.com":[{"state":"error",
        "error":"Get \"https://api.github.com/\": proxyconnect tcp: dial tcp 127.0.0.1:9: connect: connection refused",
        "active":true,"host":"github.com","login":"someone","tokenSource":"keyring",
        "gitProtocol":"https"}]}}"#;

    /// Captured with `GH_TOKEN` set to a token GitHub does not know.
    const REJECTED: &str = r#"{"hosts":{"github.com":[{"state":"error",
        "error":"non-200 OK status code: 401 Unauthorized body: \"{\r\n  \"message\": \"Bad credentials\"\r\n}\"",
        "active":true,"host":"github.com","login":"","tokenSource":"GH_TOKEN",
        "gitProtocol":"https"}]}}"#;

    const LOGGED_OUT: &str = r#"{"hosts":{}}"#;

    #[test]
    fn a_validated_credential_is_valid() {
        assert_eq!(parse(SUCCESS, Some("github.com")), Credential::Valid);
        assert_eq!(parse(SUCCESS, None), Credential::Valid);
    }

    /// **The defect, pinned.** A good credential on a machine with no network is not
    /// "logged out", and must never be answered with `gh auth login`.
    #[test]
    fn a_credential_that_could_not_be_checked_is_never_absent() {
        let state = parse(OFFLINE, Some("github.com"));
        assert!(
            matches!(state, Credential::Unverified(_)),
            "offline must not read as logged out — got {state:?}"
        );
        let Credential::Unverified(reason) = state else {
            unreachable!()
        };
        assert!(reason.contains("connection refused"), "{reason}");
    }

    /// A rejection is a verdict, and only the server can pass it.
    #[test]
    fn a_401_is_a_rejection_not_a_network_failure() {
        assert!(matches!(
            parse(REJECTED, Some("github.com")),
            Credential::Rejected(_)
        ));
    }

    #[test]
    fn no_stored_credential_is_absent() {
        assert_eq!(parse(LOGGED_OUT, Some("github.com")), Credential::Absent);
        assert_eq!(parse(LOGGED_OUT, None), Credential::Absent);
    }

    /// A host that answered without judging the credential is unverified — never a login
    /// prompt, because logging in again would not have helped.
    #[test]
    fn a_500_is_not_read_as_a_rejection() {
        let body = r#"{"hosts":{"github.com":[{"state":"error",
            "error":"non-200 OK status code: 502 Bad Gateway","active":true}]}}"#;
        assert!(matches!(
            parse(body, Some("github.com")),
            Credential::Unverified(_)
        ));
    }

    /// A host `gh` knows nothing about. Asked for explicitly, so its absence is the
    /// answer rather than a reason to fall back to another host's account.
    #[test]
    fn an_unknown_host_is_absent_rather_than_another_hosts_state() {
        assert_eq!(parse(SUCCESS, Some("ghe.example.com")), Credential::Absent);
    }
}

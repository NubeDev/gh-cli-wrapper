//! Can this operator see a repository, and does it exist at all?
//!
//! # The fact that shapes this whole file
//!
//! **GitHub returns `404` for a private repository the token cannot read** — byte for
//! byte the same response as one that does not exist. Verified against the real API
//! (gh 2.45.0):
//!
//! ```text
//! $ gh api repos/NubeIO/definitely-not-real-xyz -i   ->  HTTP/2.0 404 Not Found
//! $ gh api repos/<private, no access> -i             ->  HTTP/2.0 404 Not Found
//! ```
//!
//! That is deliberate on GitHub's side: a `403` would confirm the existence of a private
//! repo to someone who cannot see it. So no amount of care with the repo endpoint
//! separates "renamed or never created" from "you cannot read it".
//!
//! They are still two different facts with two different remedies, so both are kept — but
//! `Unreadable` is claimed only on **positive evidence that the repo exists** (the caller
//! says it is cloned locally, or GitHub admitted a 403), and otherwise the answer is
//! `Missing`, which a caller should word as "missing, renamed, or not visible to you".
//!
//! `gh repo view` is not used for this at all: it goes through GraphQL, which answers
//! both cases with the same "Could not resolve to a Repository" string and gives no
//! status code to read.

use serde::Serialize;

use crate::Error;

/// What we could establish about one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reach {
    /// Read successfully. Carries the default branch, which `show` compares a checkout
    /// against, and the repo's own web URL.
    ///
    /// `html_url` is **GitHub's field** — the REST repo object already carries it, so the
    /// link costs no extra call and no guess about the host. `None` only if GitHub stopped
    /// sending it.
    Ok {
        default_branch: Option<String>,
        html_url: Option<String>,
    },
    /// `404`, with no evidence the repo exists.
    Missing,
    /// `404`, but the repo demonstrably exists — it is cloned on disk. A permission
    /// problem, not a manifest problem.
    Unreadable,
    /// We never asked, or could not: offline, or the run stopped early. **Never
    /// collapsed into `Missing`** — "not there" and "we did not look" are the two
    /// answers a person acts on most differently.
    Unknown,
}

/// Ask GitHub about one `owner/name`.
///
/// `exists_locally` is the positive evidence that decides `Missing` from `Unreadable` on
/// a 404 — pass whether the repo is cloned in this workspace with a remote naming it.
///
/// Offline is [`Reach::Unknown`], never `Missing`. Reporting a repo as gone because the
/// network is down is the invisible failure this module exists to prevent.
#[track_caller]
pub fn reach(repo: &str, exists_locally: bool) -> Result<Reach, Error> {
    // The REST endpoint, not `gh repo view`: GraphQL cannot tell 404 from 403 (see the
    // module doc).
    let path = format!("repos/{repo}");
    let output = crate::call::output(crate::call::command().args(["api", "-i", &path]))?;
    crate::call::finished(&["api", "-i", &path], &output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if crate::rate_limit::is_network_error(&stderr) {
        return Ok(Reach::Unknown);
    }
    if let Some(limited) = crate::rate_limit::parse(&stdout) {
        return Err(limited);
    }

    Ok(classify(&stdout, exists_locally))
}

/// Decide a [`Reach`] from a `gh api -i` response — status line, headers and body.
///
/// Split from the process so every case is tested against a real captured response.
pub fn classify(response: &str, exists_locally: bool) -> Reach {
    let status = status_code(response);
    match status {
        Some(200) => Reach::Ok {
            default_branch: default_branch(response),
            html_url: body_string(response, "html_url"),
        },
        Some(404) if exists_locally => Reach::Unreadable,
        Some(404) => Reach::Missing,
        // A 403 that is not a rate limit is an access refusal that GitHub was willing to
        // admit to — SSO, or an org policy. That *is* positive evidence of existence.
        Some(403) => Reach::Unreadable,
        // Anything else — 5xx, an unparseable response — is "we did not learn", which is
        // Unknown. Guessing here is how a healthy repo gets reported as gone.
        _ => Reach::Unknown,
    }
}

/// `HTTP/2.0 404 Not Found` -> `404`.
fn status_code(response: &str) -> Option<u16> {
    response
        .lines()
        .find(|line| line.starts_with("HTTP/"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

/// The body's `default_branch`, which `show` compares a checkout's branch against.
fn default_branch(response: &str) -> Option<String> {
    body_string(response, "default_branch")
}

/// One top-level string out of a `gh api -i` response body.
///
/// Extracted when `html_url` became the second caller: two hand-rolled body splits is two
/// places to get the header/body boundary wrong, and the boundary is the fiddly part.
fn body_string(response: &str, key: &str) -> Option<String> {
    // `-i` prints headers, a blank line, then the body.
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, b)| b)
        .or_else(|| response.find("\n{").map(|index| &response[index + 1..]))?;
    serde_json::from_str::<serde_json::Value>(body.trim())
        .ok()?
        .get(key)?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real 404, captured from `gh api repos/NubeIO/definitely-not-real-xyz -i`.
    const NOT_FOUND: &str = "HTTP/2.0 404 Not Found\r\n\
        Content-Type: application/json; charset=utf-8\r\n\
        X-RateLimit-Remaining: 4998\r\n\
        \r\n\
        {\"message\":\"Not Found\",\"status\":\"404\"}";

    const FOUND: &str = "HTTP/2.0 200 OK\r\n\
        Content-Type: application/json; charset=utf-8\r\n\
        \r\n\
        {\"full_name\":\"cli/cli\",\"private\":false,\"default_branch\":\"trunk\",\
         \"html_url\":\"https://github.com/cli/cli\"}";

    /// **The whole point of this module.** The same 404 body means two different things,
    /// and the local clone is the only evidence that separates them.
    #[test]
    fn the_same_404_is_missing_or_unreadable_by_local_evidence() {
        assert_eq!(classify(NOT_FOUND, false), Reach::Missing);
        assert_eq!(classify(NOT_FOUND, true), Reach::Unreadable);
    }

    /// A 200 carries the default branch AND the repo's own web URL — `gh`'s field, from
    /// the response we already had. The link costs no second call and assumes no host.
    #[test]
    fn a_200_carries_the_default_branch_and_the_repos_own_url() {
        assert_eq!(
            classify(FOUND, false),
            Reach::Ok {
                default_branch: Some("trunk".into()),
                html_url: Some("https://github.com/cli/cli".into()),
            }
        );
    }

    /// A body with no `html_url` is `None`, never a composed `github.com/<full_name>`.
    /// That guess would be wrong on GHES.
    #[test]
    fn a_body_without_html_url_yields_none_not_a_guess() {
        let terse = "HTTP/2.0 200 OK\r\n\r\n{\"full_name\":\"o/n\"}";
        assert_eq!(
            classify(terse, false),
            Reach::Ok {
                default_branch: None,
                html_url: None,
            }
        );
    }

    /// A 403 GitHub was willing to admit to is evidence the repo exists, clone or no
    /// clone.
    #[test]
    fn a_403_is_unreadable_without_a_local_clone() {
        let forbidden = "HTTP/2.0 403 Forbidden\r\n\r\n{\"message\":\"Resource protected by SSO\"}";
        assert_eq!(classify(forbidden, false), Reach::Unreadable);
    }

    /// A 5xx or an unreadable response is `Unknown`. Reporting a healthy repo as gone
    /// because GitHub had a bad minute is the failure this guards.
    #[test]
    fn a_server_error_is_unknown_never_missing() {
        assert_eq!(
            classify("HTTP/2.0 502 Bad Gateway\r\n\r\n", true),
            Reach::Unknown
        );
        assert_eq!(classify("", false), Reach::Unknown);
        assert_ne!(classify("HTTP/2.0 500 x\r\n\r\n", false), Reach::Missing);
    }
}

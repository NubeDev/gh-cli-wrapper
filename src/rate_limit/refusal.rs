//! The last rate-limit refusal this process actually received from GitHub.
//!
//! # Why `/rate_limit` alone is not enough
//!
//! Observed live on 2026-09-23, with the same credential, one second apart: a search call
//! was refused `403` with `X-RateLimit-Remaining: 0`, `X-RateLimit-Resource: search`, and
//! `GET /rate_limit` reported the search bucket as `30/30, used 0`. The budget endpoint
//! lags the limiter. A panel built on it alone says "not blocked" to somebody who is —
//! the one wrong answer a budget panel exists to prevent.
//!
//! So [`super::parse`], which every runner calls on every failed `gh`
//! call, records what it recognised here, so an application can report it beside the
//! budget. **In memory, per process**: a server or a desktop shell lives long enough
//! to have seen one; a one-shot command has not, and reports none. Nothing is
//! persisted — it is an observation of this session, not a fact about GitHub worth a table.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// One refusal, as GitHub worded it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Refusal {
    /// Unix seconds when it was received.
    pub at: i64,
    /// `X-RateLimit-Resource` — `core`, `graphql`, `search` — when `gh` echoed the headers.
    /// Most runners see only stderr, so this is usually absent, and absent is not guessed.
    pub resource: Option<String>,
    /// `X-RateLimit-Reset`, Unix seconds, under the same condition.
    pub reset: Option<i64>,
    /// GitHub's first sentence — "API rate limit already exceeded for user ID …".
    pub message: String,
}

/// How long a refusal with no reset stays worth showing: one primary window.
const UNDATED_FOR_SECS: i64 = 3600;

static LAST: Mutex<Option<Refusal>> = Mutex::new(None);

/// Remember a refusal. Called by [`super::parse`] only.
pub(super) fn record(body: &str, resource: Option<String>, reset: Option<i64>) {
    let refusal = Refusal {
        at: crate::clock::now_unix() as i64,
        resource,
        reset,
        message: first_sentence(body),
    };
    if let Ok(mut last) = LAST.lock() {
        *last = Some(refusal);
    }
}

/// The last refusal, while it still matters at `now`: until its reset when GitHub said
/// one, otherwise for one window after it arrived.
pub fn last_refusal(now: i64) -> Option<Refusal> {
    let last = LAST.lock().ok()?.clone()?;
    current(last, now)
}

fn current(refusal: Refusal, now: i64) -> Option<Refusal> {
    let until = refusal.reset.unwrap_or(refusal.at + UNDATED_FOR_SECS);
    (until > now).then_some(refusal)
}

/// The sentence carrying the refusal, without the support boilerplate GitHub appends.
fn first_sentence(body: &str) -> String {
    let line = body
        .lines()
        .map(|l| l.trim().trim_start_matches("gh:").trim())
        .map(|l| {
            l.trim_start_matches("\"message\":")
                .trim()
                .trim_matches('"')
        })
        .map(|l| l.trim_start_matches("HTTP 403:").trim())
        .find(|l| l.to_ascii_lowercase().contains("rate limit"))
        .unwrap_or("rate limited");
    match line.find(". ") {
        Some(end) => line[..=end].to_string(),
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(at: i64, reset: Option<i64>) -> Refusal {
        Refusal {
            at,
            resource: None,
            reset,
            message: String::new(),
        }
    }

    #[test]
    fn a_dated_refusal_lasts_until_its_reset() {
        assert!(current(at(100, Some(160)), 159).is_some());
        assert!(current(at(100, Some(160)), 160).is_none());
    }

    #[test]
    fn an_undated_refusal_lasts_one_window() {
        assert!(current(at(100, None), 100 + 3599).is_some());
        assert!(current(at(100, None), 100 + 3600).is_none());
    }

    /// The operator's stderr, verbatim, and the body `gh api -i` prints — both reduce to
    /// GitHub's one sentence, without the request-id boilerplate.
    #[test]
    fn the_message_is_githubs_first_sentence() {
        assert_eq!(
            first_sentence("gh: API rate limit already exceeded for user ID 18637436."),
            "API rate limit already exceeded for user ID 18637436."
        );
        assert_eq!(
            first_sentence(
                "HTTP/2.0 403 Forbidden\n\t\"message\": \"API rate limit exceeded for user ID 1. \
                 If you reach out to GitHub Support …\","
            ),
            "API rate limit exceeded for user ID 1."
        );
        // How `gh search` words it — seen live through `ghw serve` on 2026-09-23.
        assert_eq!(
            first_sentence("HTTP 403: API rate limit exceeded for user ID 1. If you reach …"),
            "API rate limit exceeded for user ID 1."
        );
    }
}

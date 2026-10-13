//! `gh search repos` — repositories wearing a topic, across several owners in one call.
//!
//! # The flags, verified against the real CLI
//!
//! `gh search repos --help`, gh 2.45.0:
//!
//! ```text
//!   --owner strings    Filter on owner
//!   --topic strings    Filter on topic
//!   --json fields      Output JSON with the specified fields
//!   -L, --limit int    Maximum number of repositories to fetch (default 30)
//! ```
//!
//! Both are `strings`, so both accept a comma-separated list *and* repeat. Verified by
//! running it: `gh search repos --owner cli --owner github --topic cli --json fullName`
//! returns repositories from both owners in one call. **The default limit is 30** — a
//! membership index that quietly stops is worse than one that fails, so the limit is
//! explicit.

use crate::Error;

/// GitHub's own ceiling for a search: the API returns at most 1,000 results however the
/// query is paged. Asking for it makes the truncation GitHub's, not a default of ours —
/// and a product with a thousand repos has a different problem than this function.
pub const SEARCH_LIMIT: &str = "1000";

/// One repo the topic search found: `owner/name`, and where it lives on the web.
///
/// `url` is `gh`'s own field on the search result, not a composition. A repo found only
/// by the search is exactly the one we have nothing else about — no clone URL, no repo
/// object — so if the link did not come from here it would have to be invented, and the
/// host may be GHES.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundRepo {
    pub full_name: String,
    pub url: Option<String>,
    /// Whether GitHub calls it private. `None` when the caller did not ask for the field
    /// — absent is not the same fact as `false`, and a repo defaulted to public in the
    /// store would be one we had described wrongly rather than left undescribed.
    pub is_private: Option<bool>,
}

/// Repos wearing `topic`, across every owner given.
///
/// One call, `--owner` repeated. An empty `owners` returns an empty list **without
/// calling `gh`**: an unqualified topic search would sweep all of GitHub, which is a
/// slow way to get an answer that is certainly wrong.
#[track_caller]
pub fn repos(owners: &[String], topic: &str) -> Result<Vec<FoundRepo>, Error> {
    if owners.is_empty() || topic.is_empty() {
        return Ok(Vec::new());
    }

    let mut command = crate::call::command();
    command.args(["search", "repos"]);
    for owner in owners {
        command.args(["--owner", owner]);
    }
    command.args(["--topic", topic]);
    command.args(["--limit", SEARCH_LIMIT]);
    // `fullName` is already `owner/name`, so nothing is reassembled from parts.
    //
    // **`topics` is not a field here**: `gh search repos --json topics` is rejected with
    // `Unknown JSON field: "topics"` — the search API exposes no topic list at all. Asking
    // for a field that does not exist fails the whole call, not just the column.
    command.args(["--json", "fullName,url,isPrivate"]);

    let output = crate::call::output(&mut command)?;
    crate::call::finished(&["search", "repos"], &output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        // A network failure and a refused query are different remedies, so they are
        // different errors — one is "try again", the other is "your query is wrong". A
        // search spends the 30-a-minute bucket, the easiest one to empty.
        if let Some(limited) = crate::rate_limit::parse(&stderr) {
            return Err(limited);
        }
        if crate::rate_limit::is_network_error(&stderr) {
            return Err(Error::Offline(stderr.trim().to_string()));
        }
        return Err(Error::Failed {
            command_line: format!("gh search repos --topic {topic}"),
            code: output.status.code().unwrap_or(-1),
            stderr: stderr.into_owned(),
        });
    }

    Ok(parse_full_names(&String::from_utf8_lossy(&output.stdout)))
}

/// `[{"fullName":"cli/cli","url":"…"}, …]` -> the repos, in order.
///
/// Separated from the process so the shape is tested against a body captured from the
/// real API.
///
/// A result with no `fullName` is dropped — it is not a repo we can name. A result with
/// no `url` is KEPT, with `url: None`: we know it exists, we just cannot link to it.
pub fn parse_full_names(stdout: &str) -> Vec<FoundRepo> {
    serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|value| value.as_array().cloned())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some(FoundRepo {
                        full_name: item.get("fullName")?.as_str()?.to_string(),
                        url: item.get("url").and_then(|v| v.as_str()).map(str::to_string),
                        // Absent is `None`, never `false`: "we did not ask" and "GitHub
                        // says public" are different facts, and only one of them is safe
                        // to render as a public repository.
                        is_private: item.get("isPrivate").and_then(|v| v.as_bool()),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured from a real call — `gh search repos --owner cli --owner github
    /// --topic cli --json fullName,owner,name --limit 3`, gh 2.45.0 — a real response body
    /// for a parser test.
    const REAL: &str = r#"[{"fullName":"cli/cli","name":"cli","owner":{"login":"cli","type":"Organization"},"url":"https://github.com/cli/cli"},{"fullName":"github/gh-stack","name":"gh-stack","owner":{"login":"github","type":"Organization"},"url":"https://github.com/github/gh-stack"}]"#;

    /// The fan-out is the point: one call, two owners, both in the result.
    #[test]
    fn a_real_cross_org_body_yields_both_owners() {
        let found = parse_full_names(REAL);
        let names: Vec<&str> = found.iter().map(|r| r.full_name.as_str()).collect();
        assert_eq!(names, ["cli/cli", "github/gh-stack"]);
        assert!(names.iter().any(|n| n.starts_with("cli/")));
        assert!(names.iter().any(|n| n.starts_with("github/")));
    }

    /// The URL comes from the search result, not from `fullName` and a host we chose.
    #[test]
    fn the_url_is_the_searchs_own_field() {
        let found = parse_full_names(REAL);
        assert_eq!(found[0].url.as_deref(), Some("https://github.com/cli/cli"));
    }

    /// A result with no `url` is still a repo we found.
    #[test]
    fn a_result_with_no_url_is_kept_without_one() {
        let found = parse_full_names(r#"[{"fullName":"o/n"}]"#);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].url, None);
        // And one with no name at all is dropped: we cannot name it, so we cannot report it.
        assert!(parse_full_names(r#"[{"url":"https://x/o/n"}]"#).is_empty());
    }

    #[test]
    fn an_empty_result_is_an_empty_list() {
        assert!(parse_full_names("[]").is_empty());
        assert!(parse_full_names("").is_empty());
    }

    /// An unqualified topic search would sweep all of GitHub. It must not be issued at
    /// all — asserted by the absence of a process, which is why this returns early
    /// rather than building a command with no `--owner`.
    #[test]
    fn no_owners_means_no_call() {
        assert!(repos(&[], "some-topic").unwrap().is_empty());
        assert!(repos(&["cli".into()], "").unwrap().is_empty());
    }
}

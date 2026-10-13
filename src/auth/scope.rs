//! **The fourth auth state** — authenticated, `gh` present, and the token simply cannot
//! see the thing that was asked for.
//!
//! `gh auth login` does not fix a missing scope. Folding this into
//! [`super::AuthState::Unauthenticated`] would therefore send someone to a command that
//! cannot help: the rule is not "three states" but "states with different remedies stay
//! apart", and this is a fourth remedy.
//!
//! # It is classified from a response, never from `gh auth status`
//!
//! `gh auth status` reports the scopes the token *has*; it does not know which scopes the
//! query you are about to run *needs*. So unlike the other three, this state is not
//! something [`super::state`] can return — it is read off a refusal that already
//! happened. Both refusal shapes below were captured from the real API on 2026-09-21 and
//! are committed in `tests/fixtures/`: a real response body read by a parser test.
//!
//! ```text
//! $ gh project list --owner NubeIO
//! error: your authentication token is missing required scopes [read:project]
//! To request it, run:  gh auth refresh -s read:project
//!
//! $ gh api graphql -f query='query{ organization(login:"NubeIO"){ projectV2(number:1){ id } } }'
//! {"errors":[{"type":"INSUFFICIENT_SCOPES","message":"… The 'projectV2' field requires
//!   one of the following scopes: ['read:project'], but your token has only been granted
//!   the: ['gist', 'read:org', 'repo', 'workflow'] scopes. …"}]}
//! ```
//!
//! Note what the second one does *not* say: it names the scopes the token lacks **and**
//! the scopes it holds, in two identically-shaped bracket lists. Reading the first list
//! is the whole parser; reading them both would report the granted scopes as missing.

/// A refusal for want of a scope, and the one command that fixes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingScope {
    /// The scopes the call required and the token does not have, in the order GitHub
    /// named them.
    pub scopes: Vec<String>,
    /// The command to run. Rendered by [`remedy`] so every surface says one sentence.
    pub remedy: String,
}

/// The Projects v2 scope to ask for, and the reason it is the **write** one.
///
/// GitHub's refusals ask for `read:project`, and `gh`'s own suggestion echoes that — but
/// `read:project` resolves a board's schema and then fails every `gh project item-edit`
/// with a second, differently-worded scope error. Sending someone round that loop twice is
/// the failure this module exists to prevent, so the remedy asks for `project`, which
/// subsumes it.
pub const PROJECT: &str = "project";

/// The remedy for a set of missing scopes: one `gh auth refresh`, naming them.
///
/// `read:project` is answered with `-s project` for the reason [`PROJECT`] gives. Every
/// other scope is asked for by the name GitHub used, because we have no basis to widen
/// somebody's token beyond what the call actually needed.
pub fn remedy(scopes: &[String]) -> String {
    let asked: Vec<&str> = scopes
        .iter()
        .map(|scope| match scope.as_str() {
            "read:project" | "project" => PROJECT,
            other => other,
        })
        .collect();
    let mut unique: Vec<&str> = Vec::new();
    for scope in asked {
        if !unique.contains(&scope) {
            unique.push(scope);
        }
    }
    format!("gh auth refresh -s {}", unique.join(","))
}

/// Read a missing-scope refusal out of `gh`'s stderr or a GraphQL error body.
///
/// `None` means this failure is something else — which is the answer that matters most:
/// a classifier that guessed would turn a deleted board into a scope problem and send
/// someone to change a token over a board that is gone.
pub fn classify(message: &str) -> Option<MissingScope> {
    let scopes = required_scopes(message)?;
    if scopes.is_empty() {
        return None;
    }
    let remedy = remedy(&scopes);
    Some(MissingScope { scopes, remedy })
}

/// The scopes named as **required** — the first bracket list after the marker phrase, and
/// never the second one, which is what the token already holds.
fn required_scopes(message: &str) -> Option<Vec<String>> {
    // `gh`'s own CLI wording. Square brackets, comma-separated, unquoted.
    if let Some(rest) = message
        .split_once("missing required scopes")
        .map(|(_, r)| r)
    {
        return Some(names_in_first_bracket(rest, '[', ']'));
    }
    // The GraphQL body. Square brackets, single-quoted names. The phrase is repeated
    // once per refused field, so the first occurrence answers for all of them.
    if let Some(rest) = message
        .split_once("requires one of the following scopes:")
        .map(|(_, r)| r)
    {
        return Some(names_in_first_bracket(rest, '[', ']'));
    }
    None
}

/// The comma-separated names inside the first `open`…`close` pair, stripped of the quotes
/// the GraphQL form puts round each one.
fn names_in_first_bracket(text: &str, open: char, close: char) -> Vec<String> {
    let Some(start) = text.find(open) else {
        return Vec::new();
    };
    let rest = &text[start + open.len_utf8()..];
    let Some(end) = rest.find(close) else {
        return Vec::new();
    };
    rest[..end]
        .split(',')
        .map(|name| name.trim().trim_matches('\'').trim_matches('"').trim())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two real bodies, read from disk rather than pasted, so the test and the
    /// committed capture cannot drift apart.
    fn captured(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|why| panic!("{}: {why}", path.display()))
    }

    /// **`gh`'s own refusal**, captured from `gh project list --owner <org>`.
    #[test]
    fn the_cli_refusal_names_the_scope_and_the_remedy() {
        let missing = classify(&captured("missing-scope-stderr.txt")).expect("classified");
        assert_eq!(missing.scopes, ["read:project"]);
        // Not `-s read:project`, which would resolve the schema and fail every write.
        assert_eq!(missing.remedy, "gh auth refresh -s project");
    }

    /// **The GraphQL refusal**, captured from `gh api graphql` against a real project.
    /// The body names the required scopes and the granted ones in two identical-looking
    /// bracket lists; only the first is the answer.
    #[test]
    fn the_graphql_refusal_reads_the_required_scopes_not_the_granted_ones() {
        let missing = classify(&captured("insufficient-scopes-graphql.json")).expect("classified");
        assert_eq!(
            missing.scopes,
            ["read:project"],
            "the granted scopes — gist, read:org, repo, workflow — are the SECOND list \
             and must never be reported as missing"
        );
        assert_eq!(missing.remedy, "gh auth refresh -s project");
    }

    /// Anything else is **not** this state. A classifier that guessed would send someone
    /// to change a scope over a board that had been deleted.
    #[test]
    fn an_unrelated_failure_is_not_a_scope_problem() {
        assert!(classify("Could not resolve to a ProjectV2 with the number 99").is_none());
        assert!(classify("dial tcp: lookup api.github.com: no such host").is_none());
        assert!(classify("").is_none());
        // The marker phrase with nothing bracketed after it: no scope named, so no claim.
        assert!(classify("your authentication token is missing required scopes").is_none());
    }

    /// A scope that is not a project scope is asked for by the name GitHub used — we
    /// have no basis for widening a token past what the call needed.
    #[test]
    fn a_non_project_scope_is_asked_for_verbatim() {
        let missing = classify("missing required scopes [delete_repo]").expect("classified");
        assert_eq!(missing.remedy, "gh auth refresh -s delete_repo");
    }

    /// Several scopes become one `gh auth refresh`, deduplicated after `read:project` is
    /// widened — two commands to run is two chances to run one of them.
    #[test]
    fn several_scopes_become_one_command() {
        let missing = classify("missing required scopes [read:project, project, admin:org]")
            .expect("classified");
        assert_eq!(missing.remedy, "gh auth refresh -s project,admin:org");
    }
}

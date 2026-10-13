//! Which of GitHub's budgets a call draws on, and how it ended.

use serde::{Deserialize, Serialize};

/// Which of GitHub's budgets a call draws on, from its argv.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Budget {
    /// `gh api graphql`.
    Graphql,
    /// `gh api <rest path>`, and `gh auth status` (it checks the token against the API).
    Core,
    /// `gh api search/…`.
    Search,
    /// A `gh` subcommand (`repo list`, `issue edit`, `project …`): it spends GraphQL or REST
    /// inside `gh`, and never reports which or how much.
    Cli,
    /// `gh api rate_limit`, `gh --version`: costs nothing.
    Free,
}

impl Budget {
    /// Classify an argv — the arguments after `gh`.
    pub fn of(args: &[&str]) -> Budget {
        match args {
            ["api", "graphql", ..] => Budget::Graphql,
            ["api", rest @ ..] => {
                let path = rest
                    .iter()
                    .find(|a| !a.starts_with('-'))
                    .copied()
                    .unwrap_or("");
                let path = path.trim_start_matches('/');
                if path.starts_with("rate_limit") {
                    Budget::Free
                } else if path.starts_with("search/") {
                    Budget::Search
                } else {
                    Budget::Core
                }
            }
            ["auth", "status", ..] => Budget::Core,
            ["--version", ..] | ["version", ..] => Budget::Free,
            _ => Budget::Cli,
        }
    }
}

/// How the call ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ok,
    /// GitHub refused it for the rate limit — it cost nothing.
    RateLimited,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_argv_names_its_budget() {
        assert_eq!(
            Budget::of(&["api", "graphql", "-f", "query=…"]),
            Budget::Graphql
        );
        assert_eq!(Budget::of(&["api", "repos/o/n"]), Budget::Core);
        assert_eq!(Budget::of(&["api", "-i", "/search/issues"]), Budget::Search);
        assert_eq!(Budget::of(&["api", "rate_limit"]), Budget::Free);
        assert_eq!(Budget::of(&["--version"]), Budget::Free);
        assert_eq!(Budget::of(&["auth", "status"]), Budget::Core);
        assert_eq!(Budget::of(&["repo", "list"]), Budget::Cli);
    }
}

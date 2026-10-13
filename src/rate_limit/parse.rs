//! Recognise a rate-limit refusal in a `gh` response and read its budget and reset.
//!
//! Its tests read a real 403 body and a real header block, committed once as fixtures. That
//! is parsing a recorded *response shape*, not mocking the API: no test takes a code path
//! believing it reached GitHub when it did not.

use crate::Error;

/// Recognise a rate-limit failure in a `gh` response body, returning the error carrying the
/// budget and reset time. `None` when the body is some other failure.
///
/// `gh` prints the body and, on a failure, the headers it saw. Both are searched: the budget
/// lives only in the `x-ratelimit-*` headers, while the "rate limit exceeded" wording lives
/// only in the body — so neither alone identifies the failure *and* reports the budget.
///
/// **It also remembers what it recognised** ([`super::refusal`]), because `GET /rate_limit`
/// was seen reporting a full bucket while the same bucket refused the call.
///
/// A GraphQL refusal carries no headers, so no reset: that one is asked of `/rate_limit`,
/// which costs no budget ([`graphql_reset`]).
pub fn parse(body: &str) -> Option<Error> {
    if !is_rate_limited(body) {
        return None;
    }
    super::refusal::record(
        body,
        header_str(body, "x-ratelimit-resource"),
        header_str(body, "x-ratelimit-reset").and_then(|r| r.parse().ok()),
    );
    let mut reset = header_str(body, "x-ratelimit-reset");
    if reset.is_none() && is_graphql(body) {
        reset = graphql_reset();
    }
    Some(Error::RateLimited {
        remaining: header(body, "x-ratelimit-remaining").unwrap_or(0),
        limit: header(body, "x-ratelimit-limit").unwrap_or(0),
        reset: reset.unwrap_or_else(|| "unknown".into()),
    })
}

/// Whether this failure is a rate limit rather than some other 403. Pure: records nothing.
///
/// A secondary limit ("you have exceeded a secondary rate limit") is included: the remedy is
/// the same — stop, and come back later.
///
/// **"rate limit" and "exceeded" are matched apart, not as one phrase.** GitHub words the
/// primary limit two ways — `API rate limit exceeded for …` on the request that crosses it,
/// and `API rate limit already exceeded for user ID …` on every request after.
pub fn is_rate_limited(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    (lower.contains("rate limit") && lower.contains("exceeded"))
        || (lower.contains("403") && lower.contains("x-ratelimit-remaining: 0"))
}

/// Whether a refusal is GraphQL's: `gh api graphql`'s JSON body (`"type":"RATE_LIMIT"`), or
/// the `GraphQL:` line every other `gh` command that uses GraphQL prints — `gh repo list`,
/// captured with the budget spent:
/// `GraphQL: API rate limit already exceeded for user ID 18637436.`
pub fn is_graphql(body: &str) -> bool {
    body.contains("\"RATE_LIMIT\"") || body.lines().any(|l| l.trim_start().starts_with("GraphQL:"))
}

/// The `graphql` bucket's reset, Unix seconds, from `GET /rate_limit` — which costs nothing.
/// `None` when it did not answer.
#[track_caller]
pub fn graphql_reset() -> Option<String> {
    let args = ["api", "rate_limit", "--jq", ".resources.graphql.reset"];
    let output = crate::call::command().args(args).output().ok()?;
    crate::call::finished(&args, &output);
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (output.status.success() && text.parse::<i64>().is_ok()).then_some(text)
}

/// Header names are matched case-insensitively: GitHub sends `X-RateLimit-Remaining` and
/// `gh` may echo it in any case.
fn header_str(body: &str, name: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim()
            .eq_ignore_ascii_case(name)
            .then(|| value.trim().to_string())
    })
}

fn header(body: &str, name: &str) -> Option<u32> {
    header_str(body, name)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real 403 body, committed once.
    const BODY: &str = include_str!("../../tests/fixtures/rate-limit-403.json");

    /// The header block captured live from `gh api rate_limit --include`.
    const HEADERS: &str = include_str!("../../tests/fixtures/rate-limit-headers.txt");

    #[test]
    fn a_rate_limit_body_with_headers_reports_the_budget_and_reset() {
        let response = format!(
            "HTTP/2.0 403 Forbidden\nX-RateLimit-Limit: 5000\nX-RateLimit-Remaining: 0\n\
             X-RateLimit-Reset: 1788933056\n\n{BODY}"
        );
        match parse(&response) {
            Some(Error::RateLimited {
                remaining,
                limit,
                reset,
            }) => {
                assert_eq!(remaining, 0);
                assert_eq!(limit, 5000);
                assert_eq!(reset, "1788933056");
            }
            other => panic!("expected a rate limit, got {other:?}"),
        }
    }

    /// The header names and formats are the ones GitHub really sends.
    #[test]
    fn the_captured_headers_parse() {
        assert_eq!(header(HEADERS, "x-ratelimit-limit"), Some(5000));
        assert_eq!(header(HEADERS, "X-RateLimit-Limit"), Some(5000));
        assert!(header_str(HEADERS, "x-ratelimit-reset").is_some());
    }

    /// A secondary rate limit has the same remedy — stop, come back later.
    #[test]
    fn a_secondary_rate_limit_is_recognised() {
        assert!(parse("You have exceeded a secondary rate limit").is_some());
    }

    /// The wording GitHub uses on every request **after** the one that hit the wall.
    #[test]
    fn an_already_exceeded_limit_is_recognised() {
        let stderr = "gh: API rate limit already exceeded for user ID 18637436.";
        assert!(matches!(parse(stderr), Some(Error::RateLimited { .. })));
        assert!(parse("API rate limit exceeded for user ID 18637436.").is_some());
    }

    /// Some other failure is not a rate limit, and must not be reported as one.
    #[test]
    fn another_403_is_not_a_rate_limit() {
        assert!(parse("HTTP 403: Must have admin rights to Repository.").is_none());
        assert!(parse("HTTP 404: Not Found").is_none());
    }

    #[test]
    fn a_graphql_refusal_is_told_from_a_rest_one() {
        assert!(is_graphql(
            "GraphQL: API rate limit already exceeded for user ID 18637436."
        ));
        assert!(is_graphql(
            r#"{"errors":[{"type":"RATE_LIMIT","code":"graphql_rate_limit"}]}"#
        ));
        assert!(!is_graphql(
            "gh: API rate limit already exceeded for user ID 1."
        ));
        assert!(!is_graphql("You have exceeded a secondary rate limit"));
    }
}

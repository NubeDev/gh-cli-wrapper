//! `gh api graphql` — one query, its variables, and cursor pagination.
//!
//! # The flags, verified against the real CLI
//!
//! `gh api --help`, gh 2.45.0:
//!
//! - The endpoint is the literal word `graphql`.
//! - `-f/--raw-field key=value` adds a **string** parameter; `-F/--field` applies magic
//!   type conversion (`true`, `null`, integers, and a leading `@` meaning "read a file").
//!   The query itself goes through `-f`, always: a query is a string, and `-F` would
//!   reinterpret one that happened to start with `@` as a filename.
//! - "For GraphQL requests, all fields other than `query` and `operationName` are
//!   interpreted as GraphQL variables" — so variables are passed as fields, not encoded
//!   into a JSON `variables` object.
//! - `--paginate` "requires that the original query accepts an `$endCursor: String`
//!   variable and that it fetches the `pageInfo{ hasNextPage, endCursor }` set of
//!   fields". That is a property of the caller's query, not of this function, so
//!   [`graphql`] refuses `paginate` on a query that does not carry both rather than
//!   returning one silent page.
//!
//! # Two errors GraphQL has that REST does not
//!
//! A GraphQL request can return **HTTP 200 with an `errors` array** and a partial
//! `data`. Treating that as success is how a caller ends up reporting a partial answer
//! as a complete one, so `errors` is an error here even when the process exited 0.

use crate::Error;

use super::errors::graphql_errors;

/// Run a GraphQL query. `vars` must be a JSON object; each key becomes a `-F` field,
/// which is how `gh` passes GraphQL variables.
///
/// With `paginate`, `gh` walks `pageInfo` itself and concatenates the pages — each page
/// arrives as its own JSON document, so the result is a JSON **array** of page objects
/// rather than one merged object. That difference is the caller's to handle, and saying
/// so here is cheaper than a caller discovering it against a two-page product.
#[track_caller]
pub fn graphql(
    query: &str,
    vars: &serde_json::Value,
    paginate: bool,
) -> Result<serde_json::Value, Error> {
    if paginate && !(query.contains("endCursor") && query.contains("hasNextPage")) {
        // Fail rather than return page one and call it the answer — `gh` itself would
        // silently stop, which is an invisible failure.
        return Err(Error::Failed {
            command_line: "gh api graphql --paginate".into(),
            code: -1,
            stderr: "a --paginate query must accept $endCursor and select \
                     pageInfo{ hasNextPage, endCursor }; this one does not, so gh would \
                     return only the first page"
                .into(),
        });
    }

    let output = spawn(query, vars, paginate)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(failure(&stdout, &stderr, output.status.code()));
    }

    let value: serde_json::Value = serde_json::from_str(stdout.trim())?;
    if let Some(message) = graphql_errors(&value) {
        return Err(Error::Failed {
            command_line: "gh api graphql".into(),
            code: 0,
            stderr: message,
        });
    }
    Ok(value)
}

/// **A batched query whose aliases may fail one at a time** — the body as GitHub sent it,
/// `errors` and all. `gh` exits 1 when any alias failed (`NOT_FOUND` on one repository of
/// twenty-five), yet the other twenty-four answered in `data`; [`graphql`] would throw them
/// away with the one. The caller reads `data` per alias and treats a `null` as unknown.
///
/// Anything that is not a GraphQL body — a rate limit, the network, no `gh` — is still an
/// error, exactly as [`graphql`] reports it.
#[track_caller]
pub fn graphql_partial(query: &str, vars: &serde_json::Value) -> Result<serde_json::Value, Error> {
    let output = spawn(query, vars, false)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let body = serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .filter(|v| v.get("data").is_some_and(|d| !d.is_null()));
    match body {
        Some(body) if crate::rate_limit::parse(&stdout).is_none() => Ok(body),
        _ if output.status.success() => Err(Error::Failed {
            command_line: "gh api graphql".into(),
            code: 0,
            stderr: "GitHub answered with no data".into(),
        }),
        _ => Err(failure(&stdout, &stderr, output.status.code())),
    }
}

/// Run `gh api graphql` once, logged by the call log.
#[track_caller]
fn spawn(
    query: &str,
    vars: &serde_json::Value,
    paginate: bool,
) -> Result<std::process::Output, Error> {
    let mut command = crate::call::command();
    command.args(["api", "graphql"]);
    // `-f` and not `-F`: the query is a string, and `-F` would read a query beginning
    // with `@` as a filename.
    // `rateLimit { cost }` rides along, so the call log holds what GitHub charged.
    command.args(["-f", &format!("query={}", super::cost::with_cost(query))]);

    if let Some(object) = vars.as_object() {
        for (key, value) in object {
            for (flag, field) in fields(key, value) {
                command.args([flag, &field]);
            }
        }
    }
    if paginate {
        command.arg("--paginate");
    }

    let output = crate::call::output(&mut command)?;
    crate::call::finished(&["api", "graphql"], &output);
    Ok(output)
}

/// A `gh api graphql` that exited non-zero, as the error it is.
fn failure(stdout: &str, stderr: &str, code: Option<i32>) -> Error {
    if let Some(limited) = crate::rate_limit::parse(stdout) {
        return limited;
    }
    if crate::rate_limit::is_network_error(stderr) {
        return Error::Offline(stderr.trim().to_string());
    }
    Error::Failed {
        command_line: "gh api graphql".into(),
        code: code.unwrap_or(-1),
        stderr: stderr.to_string(),
    }
}

/// One variable as `gh api` fields: `(flag, key=value)`.
///
/// **A string goes with `-f`, everything else with `-F`** (`gh api --help`, gh 2.101.0).
/// `-F`'s type conversion is what a number, a boolean and `null` need, and what a string
/// must not get: it reads a value starting with `@` as a file name, turns `true` or `123`
/// into JSON, and fills `{owner}` / `{repo}` / `{branch}` in from the current checkout — so
/// a check-in note, a README or an issue title was being rewritten on its way to GitHub.
/// An **object** is `key[sub]=value`, gh's nested syntax, which both flags take: a
/// serialised object arrives at GitHub as a *string*, which an input type such as
/// `CommitAuthor` refuses — "provided invalid value", observed live on 2026-09-25. Arrays
/// keep their serialised form.
fn fields(key: &str, value: &serde_json::Value) -> Vec<(&'static str, String)> {
    match value {
        serde_json::Value::Object(object) => object
            .iter()
            .flat_map(|(sub, inner)| fields(&format!("{key}[{sub}]"), inner))
            .collect(),
        serde_json::Value::String(s) => vec![("-f", format!("{key}={s}"))],
        other => vec![("-F", format!("{key}={other}"))],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An input object must reach gh as nested fields, not as a JSON string.
    #[test]
    fn an_object_variable_is_gh_nested_fields() {
        let id = serde_json::json!({ "id": "MDQ6VXNlcjY4MDA3NzU=" });
        let f = |k: &str, v: serde_json::Value| fields(k, &v);
        assert_eq!(
            f("author", id),
            [("-f", "author[id]=MDQ6VXNlcjY4MDA3NzU=".to_string())]
        );
        assert_eq!(
            f("author", serde_json::Value::Null),
            [("-F", "author=null".to_string())]
        );
        assert_eq!(f("n", serde_json::json!(50)), [("-F", "n=50".to_string())]);
        assert_eq!(
            f("q", serde_json::json!("org:x is:pr")),
            [("-f", "q=org:x is:pr".to_string())]
        );
    }

    /// A string reaches GitHub as written — never read as a file, a number, a boolean or a
    /// placeholder, which `-F` would do to it.
    #[test]
    fn a_string_that_looks_like_something_else_is_still_a_string() {
        for text in [
            "@team please look",
            "123",
            "true",
            "ships to {repo} on {branch}",
        ] {
            assert_eq!(
                fields("body", &serde_json::json!(text)),
                [("-f", format!("body={text}"))],
                "{text}"
            );
        }
    }

    /// `gh --paginate` stops silently on a query with no `pageInfo`. Returning page one
    /// as though it were the whole answer is the invisible failure; this refuses instead.
    #[test]
    fn paginate_refuses_a_query_that_cannot_paginate() {
        let err = graphql("query { viewer { login } }", &serde_json::json!({}), true).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("endCursor"), "{message}");
        assert!(message.contains("first page"), "{message}");
    }

    /// And a query that *can* paginate passes the guard — the check is on the query, not
    /// a blanket refusal of `--paginate`.
    #[test]
    fn a_paginating_query_passes_the_guard() {
        let query = "query($endCursor: String) { \
                     search(first: 100, after: $endCursor) { \
                     pageInfo { hasNextPage endCursor } } }";
        // Reaching `gh` at all means the guard let it through; whether the call
        // succeeds depends on the network, which is not what this asserts.
        let result = graphql(query, &serde_json::json!({}), true);
        if let Err(err) = &result {
            assert!(
                !err.to_string().contains("endCursor"),
                "the guard rejected a query that does paginate: {err}"
            );
        }
    }
}

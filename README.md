# gh-cli-wrapper

A reusable Rust library for building local tools that use the user's existing GitHub CLI setup.

This is not a GitHub REST SDK. It discovers hosts from `gh` configuration and delegates
authenticated work to the `gh` executable, so your app does not need to read or store tokens.

```rust,no_run
use gh_cli_wrapper::GhContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Viewer {
    login: String,
}

fn main() -> Result<(), gh_cli_wrapper::Error> {
    let ctx = GhContext::load()?;
    let host = ctx.default_host().map(|host| host.hostname.as_str());
    let viewer: Viewer = ctx.gh().api_json(host, "user")?;

    println!("{}", viewer.login);
    Ok(())
}
```

## Why delegate to `gh`

- **No token custody.** `gh` already holds the credential; you inherit auth, refresh, SSO and
  enterprise hosts for free.
- **Whatever the user has configured already works** — `github.com`, GHES, several hosts.

## What's in it

| Module | What it is |
|---|---|
| `call` | **The one constructor** every `gh` process is built by: `command()`, `finished()`. Optional JSON-lines call log — off until `set_log_path` names a file — with the call site (`#[track_caller]`), the budget it drew on, and what GitHub charged |
| `run` | The raw runner for a verb with no typed builder: `stdout` / `text` / `json` / `status` (and the `_as` forms naming the command line), `api_paginated`, `allowing_failure`, `rendered` |
| `auth` | `credential()` reads `gh auth status --json hosts` (Valid · Absent · Rejected · Unverified · GhMissing); `state()` projects it onto `AuthState`; `scope::classify` reads a missing-scope refusal; `held_scopes()`; `device_prompt::parse`; `login()` |
| `rate_limit` | `parse` a refusal (both of GitHub's wordings, headers, a GraphQL refusal's reset), `is_network_error`, the last refusal seen |
| `graphql` | `graphql(query, vars, paginate)`, `graphql_partial`, `graphql_errors`, `with_cost` |
| `token` | `set_ambient` / `with_token` — a `GH_TOKEN` for every `gh` process, for a server with no `gh` login |
| `search` · `repo` · `version` | `search::repos`, `repo::reach`, `repo::add_topic`, `version::installed` |
| `pr` · `issue` · `repo` · `release` · `git` | Typed command builders, one file per verb |

## Four auth states, not a bool

States with **different remedies** stay apart:

| `AuthState` | Remedy |
|---|---|
| `Authenticated` | — |
| `Unauthenticated` | `gh auth login` |
| `GhMissing` | install `gh` — `gh auth login` cannot fix this |
| `MissingScope(_)` | `gh auth refresh -s <scope>` — `gh auth login` reissues the same scopes |

A credential `gh` could not check (no network) is not an auth state: `auth::state` returns
`Error::Offline`, never `Unauthenticated`.

## Errors

`Error` keeps failures with different remedies in different variants — `GhMissing`,
`Failed { command_line, code, stderr }`, `RateLimited { remaining, limit, reset }`,
`Offline`, `MissingScope` — so a caller switches on the variant instead of parsing a message.

## Tests

`cargo test` runs the unit tests and the builder tests offline. The live test writes to a real
repository and is opt-in: `LIVE_GITHUB_TESTS=1 cargo test -- --ignored`.

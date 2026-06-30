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

//! `gh-cli-wrapper` — reuse the user's GitHub CLI setup from Rust.
//!
//! This crate is intentionally not a GitHub REST SDK. It discovers local `gh` configuration with
//! `gh-config`, then delegates authenticated work to the `gh` executable so applications do not
//! need to read, store, or expose tokens.
//!
//! - [`call`] — the one constructor every `gh` process is built by, and its call log.
//! - [`run`] — the raw runner, which classifies a failure by its remedy.
//! - [`auth`] — the credential `gh` holds, the four auth states, scopes, the login.
//! - [`rate_limit`] — GitHub's limit read off a response, and network failures.
//! - [`graphql`] — `gh api graphql`, its errors and its cost.
//! - [`token`] — a token for every `gh` process, for a server with no `gh` login.
//! - [`search`], [`repo`], [`issue`], [`pr`], [`release`], [`git`] — verbs.

mod args;
mod clock;
mod command;
mod context;
mod error;
mod host;

pub mod auth;
pub mod call;
pub mod git;
pub mod graphql;
pub mod issue;
pub mod pr;
pub mod rate_limit;
pub mod release;
pub mod repo;
pub mod run;
pub mod search;
pub mod token;
pub mod version;

pub use auth::AuthState;
pub use command::{Gh, GhOutput};
pub use context::GhContext;
pub use error::Error;
pub use host::GhHost;

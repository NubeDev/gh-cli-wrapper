//! `gh-cli-wrapper` — reuse the user's GitHub CLI setup from Rust.
//!
//! This crate is intentionally not a GitHub REST SDK. It discovers local `gh` configuration with
//! `gh-config`, then delegates authenticated work to the `gh` executable so applications do not
//! need to read, store, or expose tokens.

mod args;
mod auth;
mod command;
mod context;
mod error;
mod host;

pub mod git;
pub mod issue;
pub mod pr;
pub mod release;
pub mod repo;

pub use auth::AuthState;
pub use command::{Gh, GhOutput};
pub use context::GhContext;
pub use error::Error;
pub use host::GhHost;

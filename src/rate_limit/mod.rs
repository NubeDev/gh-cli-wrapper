//! GitHub's rate limit, read off a `gh` response — and the network failures that are not it.
//!
//! **Never retry into the wall.** On a limit, stop, report the remaining budget and the
//! reset, and let the caller report what completed. A network failure is different: one
//! retry, then stop.
//!
//! Barrel only.

pub mod network;
pub mod parse;
pub mod refusal;

pub use network::is_network_error;
pub use parse::{graphql_reset, is_graphql, is_rate_limited, parse};
pub use refusal::{last_refusal, Refusal};

//! `gh api graphql`: run a query, read its errors, and ask what it cost.
//!
//! Barrel only.

pub mod cost;
pub mod errors;
pub mod query;

pub use cost::{graphql_cost, with_cost};
pub use errors::graphql_errors;
pub use query::{graphql, graphql_partial};

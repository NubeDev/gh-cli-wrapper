//! `gh search …` verbs.
//!
//! Barrel only.

pub mod repos;

pub use repos::{parse_full_names, repos, FoundRepo, SEARCH_LIMIT};

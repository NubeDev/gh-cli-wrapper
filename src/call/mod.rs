//! **Every `gh` process this crate starts is built here** — and, when the application has
//! said where, leaves a line in its call log.
//!
//! [`command`] is `#[track_caller]`, and so is every runner in the crate that calls it, so
//! the site recorded is the **application's** call site (`sprint/board_read.rs:139`), not
//! this crate's. An application that wraps [`command`] in its own `#[track_caller]` function
//! keeps that property: the location chains through.
//!
//! Two lines per call, when a log path is set ([`set_log_path`]):
//!
//! - a [`Line`] at construction — the process and the site, **never the arguments** (no
//!   query, no repository name). A command built and not run is already a bug worth seeing.
//! - a [`Finished`] after it, from [`finished`] — which budget it drew on, whether GitHub
//!   answered, refused it for the rate limit, or it failed, and what GitHub charged when it
//!   said. `None` is "GitHub did not say", never a guess.
//!
//! A write that fails is ignored — the log must never be why a `gh` call did not run.
//!
//! Barrel only.

pub mod budget;
pub mod command;
pub mod finished;
pub mod log;

pub use budget::{Budget, Outcome};
pub use command::{command, command_for, output};
pub use finished::{finished, finished_as};
pub use log::{append, log_path, rotated, set_log_path, Finished, Line, ROTATE_AT};

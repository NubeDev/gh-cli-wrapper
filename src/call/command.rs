//! The constructor: one `Command` per `gh` process, logged and carrying the token in force.

use std::cell::Cell;
use std::ffi::OsStr;
use std::panic::Location;
use std::process::{Command, Output};

use crate::Error;

thread_local! {
    /// The site of the last command built on this thread — what [`super::finished`] reports
    /// against, so a runner need not carry it. A runner builds, runs and finishes on one
    /// thread; a fan-out gives each call its own.
    static LAST_SITE: Cell<Option<&'static Location<'static>>> = const { Cell::new(None) };
}

/// A `gh` command, logged. Every `gh` process the crate starts begins here.
#[track_caller]
pub fn command() -> Command {
    command_for("gh")
}

/// [`command`] for a caller-chosen executable — a bundled `gh`, or a test's.
#[track_caller]
pub fn command_for(program: impl AsRef<OsStr>) -> Command {
    let site = Location::caller();
    LAST_SITE.with(|last| last.set(Some(site)));
    super::log::spawned(site);
    let mut command = Command::new(program);
    crate::token::apply(&mut command);
    command
}

/// Run a built command, with a missing executable as [`Error::GhMissing`] rather than a bare
/// I/O error — a different remedy from every other failure.
pub fn output(command: &mut Command) -> Result<Output, Error> {
    command.output().map_err(Error::spawn)
}

/// The site of the last [`command`] built on this thread.
pub(crate) fn last_site() -> Option<&'static Location<'static>> {
    LAST_SITE.with(Cell::get)
}

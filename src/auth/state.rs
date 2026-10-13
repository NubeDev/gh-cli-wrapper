//! The four auth states, projected from a [`Credential`] reading.

use crate::{Error, Gh};

use super::credential::{credential_with, Credential};
use super::scope::MissingScope;

/// Authentication state. **Four states, not a bool.**
///
/// [`AuthState::MissingScope`] is the one variant [`state`] never returns, and that is a
/// property rather than a gap: `gh auth status` reports the scopes a token *has* and cannot
/// know which scopes the next query *needs*. It is classified off a refusal that already
/// happened — see [`super::scope::classify`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// `gh` reported a usable authenticated session.
    Authenticated,
    /// `gh` is installed, and logged out of the host — or its credential was refused.
    Unauthenticated,
    /// The `gh` executable could not be started.
    GhMissing,
    /// Authenticated, `gh` present, and the token lacks a scope the call needed.
    MissingScope(MissingScope),
}

/// The auth state for `host` (`gh`'s default host when `None`), using `gh` from `PATH`.
///
/// `Credential::Absent` and `Credential::Rejected` both project onto
/// [`AuthState::Unauthenticated`]: different facts with the *same* remedy. A credential `gh`
/// could not check has a different remedy and so is returned as [`Error::Offline`], carrying
/// `gh`'s own reason.
///
/// Note for anything that *simulates* a logged-out `gh`: `GH_TOKEN` and `GITHUB_TOKEN` take
/// precedence over stored config, so pointing `GH_CONFIG_DIR` at an empty directory is not
/// on its own enough. Both variables have to be cleared as well.
#[track_caller]
pub fn state(host: Option<&str>) -> Result<AuthState, Error> {
    state_with(&Gh::new(), host)
}

/// [`state`] through a caller-chosen `gh`.
#[track_caller]
pub fn state_with(gh: &Gh, host: Option<&str>) -> Result<AuthState, Error> {
    match credential_with(gh, host)? {
        Credential::Valid => Ok(AuthState::Authenticated),
        Credential::Absent | Credential::Rejected(_) => Ok(AuthState::Unauthenticated),
        Credential::GhMissing => Ok(AuthState::GhMissing),
        Credential::Unverified(reason) => Err(Error::Offline(reason)),
    }
}

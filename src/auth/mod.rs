//! What `gh` can say about the credential it holds — and the four auth states it projects
//! onto.
//!
//! The rule is not a count: **states with different remedies stay apart**. Logged out is
//! fixed by `gh auth login`; `gh` missing by installing it; a token lacking a scope by
//! `gh auth refresh` (which `gh auth login` cannot do — it reissues a token with the scopes
//! it already has); and a credential `gh` could not check is not an auth state at all, it is
//! the network ([`crate::Error::Offline`]).
//!
//! We never read, store or expose a token. `gh` holds it.
//!
//! Barrel only.

pub mod account;
pub mod credential;
pub mod device_prompt;
pub mod scope;
pub mod scopes;
pub mod state;

pub use account::login;
pub use credential::{credential, credential_with, Credential};
pub use device_prompt::DevicePrompt;
pub use scope::{MissingScope, PROJECT};
pub use scopes::held_scopes;
pub use state::{state, state_with, AuthState};

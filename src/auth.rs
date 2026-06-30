/// Authentication state as observed through `gh auth status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// `gh` reported a usable authenticated session.
    Authenticated,
    /// `gh` is installed, but the requested host is not authenticated.
    Unauthenticated,
    /// The `gh` executable could not be started.
    GhMissing,
}

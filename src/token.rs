//! Whose GitHub token a `gh` process carries — **nothing, unless the application sets one**.
//!
//! By default every `gh` process uses the person's own `gh` login, which is the point of this
//! crate. A server with no `gh` login (one holding a GitHub App's installation tokens, say)
//! hands each process a token instead:
//!
//! - [`set_ambient`] — the token every `gh` process in this process carries until it is
//!   changed. Process-wide rather than per thread, because a fan-out starts threads of its
//!   own and every one of them must carry it.
//! - [`with_token`] — this thread only, for the length of one closure, and it wins over the
//!   ambient token.
//!
//! The token goes into the child's environment as `GH_TOKEN` and nowhere else: never into
//! the call log, an argv, an error or a `Debug`. The process environment itself is never
//! changed — setting a variable while other threads read it is undefined behaviour.

use std::cell::RefCell;
use std::process::Command;
use std::sync::RwLock;

static AMBIENT: RwLock<Option<String>> = RwLock::new(None);

thread_local! {
    static SCOPED: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// The token every `gh` process started from now on carries. `None` goes back to `gh`'s own
/// login.
pub fn set_ambient(token: Option<String>) {
    let mut slot = AMBIENT.write().unwrap_or_else(|e| e.into_inner());
    *slot = token;
}

/// Run `f` with every `gh` process **this thread** starts carrying `token`.
pub fn with_token<R>(token: &str, f: impl FnOnce() -> R) -> R {
    let previous = SCOPED.with(|s| s.replace(Some(token.to_string())));
    let restore = Restore(previous);
    let out = f();
    drop(restore);
    out
}

/// Whether the process-wide token is in force (and no scoped one overrides it).
pub fn ambient_in_force() -> bool {
    SCOPED.with(|s| s.borrow().is_none())
        && AMBIENT.read().unwrap_or_else(|e| e.into_inner()).is_some()
}

/// Put the token in force, if any, on a `gh` command. Called by `call::command_for` only.
pub(crate) fn apply(command: &mut Command) {
    let scoped = SCOPED.with(|s| s.borrow().clone());
    let token = scoped.or_else(|| AMBIENT.read().unwrap_or_else(|e| e.into_inner()).clone());
    if let Some(token) = token {
        command.env("GH_TOKEN", token).env_remove("GITHUB_TOKEN");
    }
}

/// Puts the previous scoped token back even if the closure panics.
struct Restore(Option<String>);

impl Drop for Restore {
    fn drop(&mut self) {
        let previous = self.0.take();
        SCOPED.with(|s| *s.borrow_mut() = previous);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gh_token(command: &Command) -> Option<String> {
        command
            .get_envs()
            .find(|(k, _)| *k == "GH_TOKEN")
            .and_then(|(_, v)| v.map(|v| v.to_string_lossy().into_owned()))
    }

    #[test]
    fn nothing_set_leaves_gh_its_own_login() {
        let mut c = Command::new("any-program");
        apply(&mut c);
        assert_eq!(gh_token(&c), None);
    }

    #[test]
    fn a_scoped_token_applies_inside_the_closure_only() {
        let inside = with_token("user-token", || {
            let mut c = Command::new("any-program");
            apply(&mut c);
            gh_token(&c)
        });
        assert_eq!(inside.as_deref(), Some("user-token"));
        let mut after = Command::new("any-program");
        apply(&mut after);
        assert_eq!(gh_token(&after), None);
    }
}

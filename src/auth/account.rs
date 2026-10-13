//! Which account `gh` is acting as.

/// The authenticated login, or `None` when `gh` is absent, logged out, or offline.
///
/// All three collapse to `None` here, deliberately: this answers "do we have a name", for
/// provenance and display. A caller that has to tell the three apart — they have different
/// remedies — asks [`super::credential`].
///
/// **Never fabricated.** An account that could not be read is `None`, not a guess.
#[track_caller]
pub fn login() -> Option<String> {
    let output = crate::call::command()
        .args(["api", "user", "--jq", ".login"])
        .output()
        .ok()?;
    crate::call::finished(&["api", "user"], &output);
    if !output.status.success() {
        return None;
    }
    let login = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!login.is_empty()).then_some(login)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name is never invented. Asserted against a `PATH` with no `gh`, which is the
    /// cheapest way to produce "we could not read it".
    #[test]
    #[cfg(unix)]
    fn an_unreadable_account_is_none_rather_than_a_name() {
        let empty = std::env::temp_dir().join(format!("ghcw-nogh-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();

        let saved = std::env::var_os("PATH");
        std::env::set_var("PATH", &empty);
        let login = login();
        match saved {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        assert_eq!(login, None);
    }
}

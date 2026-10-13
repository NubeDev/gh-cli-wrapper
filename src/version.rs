//! The `gh` on `PATH`: its version, and comparing versions the way numbers compare.

/// `gh --version` from `PATH` — `gh version 2.45.0 (2025-07-18 …)` → `2.45.0`.
///
/// `None` when `gh` is absent or printed nothing that looks like a version.
#[track_caller]
pub fn installed() -> Option<String> {
    let output = crate::call::command().arg("--version").output().ok()?;
    crate::call::finished(&["--version"], &output);
    if !output.status.success() {
        return None;
    }
    version_in(&String::from_utf8_lossy(&output.stdout))
}

/// The first token on the first line that starts with a digit and has a dot.
pub fn version_in(text: &str) -> Option<String> {
    text.lines()
        .next()?
        .split_whitespace()
        .find(|t| t.starts_with(|c: char| c.is_ascii_digit()) && t.contains('.'))
        .map(str::to_string)
}

/// Whether `found` is older than `floor`, numerically: `2.100.0` is newer than `2.94.0`,
/// which a string compare gets backwards.
pub fn is_older(found: &str, floor: &str) -> bool {
    parts(found) < parts(floor)
}

/// `2.94.0` → `[2, 94, 0]`. A non-numeric suffix (`2.94.0-rc1`) ends the number.
pub fn parts(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|p| {
            p.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_version_gh_prints() {
        let out = "gh version 2.45.0 (2025-07-18 Ubuntu 2.45.0-1ubuntu0.3)\nhttps://…";
        assert_eq!(version_in(out).as_deref(), Some("2.45.0"));
    }

    #[test]
    fn compares_numerically_not_as_text() {
        assert!(is_older("2.45.0", "2.94.0"));
        assert!(!is_older("2.94.0", "2.94.0"));
        assert!(
            !is_older("2.101.0", "2.94.0"),
            "a string compare says 2.101 < 2.94"
        );
        assert!(!is_older("v3.0.0", "2.94.0"));
    }
}

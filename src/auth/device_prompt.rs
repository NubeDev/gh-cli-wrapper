//! Read the one-time code and the device URL out of what a headless `gh auth login` or
//! `gh auth refresh` prints — so an application can show them in its own window.
//!
//! With no TTY, `gh` 2.94.0 prints exactly this to stderr and then waits — captured
//! 2026-09-25:
//!
//! ```text
//! ! First copy your one-time code: EE3F-AD70
//! Open this URL to continue in your web browser: https://github.com/login/device
//! ```
//!
//! **Both or nothing.** A code with no URL, or a URL with no code, is half a prompt; the
//! caller keeps reading rather than showing a dialog with a hole in it.

/// The code and the URL the person needs, once both have been printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePrompt {
    pub code: String,
    pub url: String,
}

/// Parse what `gh` has printed so far.
pub fn parse(stderr: &str) -> Option<DevicePrompt> {
    let text = strip_ansi(stderr);
    let code = text
        .lines()
        .filter(|line| line.contains("one-time code"))
        .flat_map(|line| line.split_whitespace())
        .find(|word| is_code(word))?
        .to_string();
    let url = text
        .split_whitespace()
        .find(|word| word.starts_with("https://"))?
        .trim_end_matches(['.', ','])
        .to_string();
    Some(DevicePrompt { code, url })
}

/// `XXXX-XXXX`, letters and digits — the shape GitHub's device codes take.
fn is_code(word: &str) -> bool {
    let bytes = word.as_bytes();
    bytes.len() == 9
        && bytes[4] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// `gh` colours the code when it believes a terminal is attached. It does not headless,
/// but a `CLICOLOR_FORCE` in someone's environment would put escapes around the one word
/// we need, so they are removed rather than assumed absent.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real headless output, verbatim.
    const REAL: &str = "\n! First copy your one-time code: EE3F-AD70\n\
                        Open this URL to continue in your web browser: https://github.com/login/device\n";

    #[test]
    fn the_real_prompt_parses_to_its_code_and_url() {
        assert_eq!(
            parse(REAL),
            Some(DevicePrompt {
                code: "EE3F-AD70".into(),
                url: "https://github.com/login/device".into(),
            })
        );
    }

    #[test]
    fn a_coloured_prompt_still_parses() {
        let coloured = "\u{1b}[0;33m!\u{1b}[0m First copy your one-time code: \
                        \u{1b}[1mEE3F-AD70\u{1b}[0m\nOpen this URL to continue in your web \
                        browser: https://github.com/login/device\n";
        assert_eq!(parse(coloured).map(|p| p.code), Some("EE3F-AD70".into()));
    }

    /// Half a prompt is not a prompt: the dialog would show a code with nowhere to type it.
    #[test]
    fn half_a_prompt_is_nothing() {
        assert_eq!(parse("! First copy your one-time code: EE3F-AD70\n"), None);
        assert_eq!(
            parse("Open this URL to continue in your web browser: https://github.com/login/device"),
            None
        );
        assert_eq!(parse(""), None);
    }

    #[test]
    fn only_the_code_shape_is_a_code() {
        assert!(is_code("EE3F-AD70"));
        assert!(!is_code("ee3f-ad70"));
        assert!(!is_code("EE3FAD70"));
        assert!(!is_code("code:"));
    }
}

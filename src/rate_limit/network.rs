//! A network-class failure in `gh`'s stderr — worth **one** retry, unlike a rate limit.

/// Recognise a network-class failure in `gh`'s stderr, so it becomes
/// [`crate::Error::Offline`] rather than an opaque failure.
pub fn is_network_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("dial tcp")
        || lower.contains("no such host")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
        || lower.contains("i/o timeout")
        || lower.contains("tls handshake timeout")
        || lower.contains("error connecting to")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_class_stderr_is_recognised() {
        for stderr in [
            "error connecting to api.github.com",
            "dial tcp 140.82.121.6:443: connect: connection refused",
            "Get \"https://api.github.com\": net/http: TLS handshake timeout",
            "lookup api.github.com: no such host",
        ] {
            assert!(is_network_error(stderr), "{stderr}");
        }
        // A real API failure is not a network failure: retrying it changes nothing.
        assert!(!is_network_error("HTTP 404: Not Found"));
        assert!(!is_network_error("HTTP 403: Must have admin rights"));
    }
}

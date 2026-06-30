/// A configured GitHub host from the user's `gh` setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhHost {
    pub hostname: String,
    pub user: Option<String>,
    pub git_protocol: Option<String>,
    pub enterprise: bool,
}

impl GhHost {
    pub(crate) fn new(hostname: String, host: &gh_config::Host) -> Self {
        Self {
            enterprise: gh_config::is_enterprise(&hostname),
            hostname,
            user: host.user.clone(),
            git_protocol: host.git_protocol.as_ref().map(|p| format!("{p:?}")),
        }
    }
}

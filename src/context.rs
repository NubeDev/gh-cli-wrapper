use std::collections::BTreeSet;

use crate::{AuthState, Error, Gh, GhHost};

/// Local GitHub CLI context for applications that want to reuse `gh`.
#[derive(Debug, Clone)]
pub struct GhContext {
    gh: Gh,
    hosts: Vec<GhHost>,
}

impl GhContext {
    /// Load local `gh` host configuration and use `gh` from `PATH`.
    pub fn load() -> Result<Self, Error> {
        Self::load_with_gh(Gh::new())
    }

    /// Load local `gh` host configuration with a caller-supplied executable.
    pub fn load_with_gh(gh: Gh) -> Result<Self, Error> {
        let hosts = load_hosts()?;
        Ok(Self { gh, hosts })
    }

    /// Return the configured hosts discovered from the GitHub CLI config directory.
    pub fn hosts(&self) -> &[GhHost] {
        &self.hosts
    }

    /// Pick `github.com` when configured, otherwise the first configured host.
    pub fn default_host(&self) -> Option<&GhHost> {
        self.host("github.com").or_else(|| self.hosts.first())
    }

    /// Find a configured host by hostname.
    pub fn host(&self, hostname: &str) -> Option<&GhHost> {
        self.hosts.iter().find(|host| host.hostname == hostname)
    }

    /// The auth state for `hostname`, read from `gh auth status --json hosts` — see
    /// [`crate::auth::state`]. A credential `gh` could not check is [`Error::Offline`], not a
    /// login problem.
    #[track_caller]
    pub fn auth_state(&self, hostname: Option<&str>) -> Result<AuthState, Error> {
        crate::auth::state_with(&self.gh, hostname)
    }

    /// Access the underlying `gh` runner for custom commands.
    pub fn gh(&self) -> &Gh {
        &self.gh
    }
}

fn load_hosts() -> Result<Vec<GhHost>, Error> {
    let hosts = gh_config::Hosts::load()?;
    let mut names = host_names_from_file()?;
    names.insert("github.com".to_string());

    Ok(names
        .into_iter()
        .filter_map(|name| hosts.get(&name).map(|host| GhHost::new(name, host)))
        .collect())
}

fn host_names_from_file() -> Result<BTreeSet<String>, Error> {
    let config_dir = gh_config::find_config_directory().ok_or(gh_config::Error::ConfigNotFound)?;
    let value: serde_yaml::Value =
        serde_yaml::from_slice(&std::fs::read(config_dir.join("hosts.yml"))?)?;

    Ok(value
        .as_mapping()
        .map(|map| {
            map.keys()
                .filter_map(|key| key.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default())
}

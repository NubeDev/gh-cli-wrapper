use std::fs;
use std::sync::{Mutex, OnceLock};

use gh_cli_wrapper::{AuthState, Gh, GhContext};
use tempfile::TempDir;

#[test]
fn context_loads_hosts_without_exposing_tokens() {
    let temp = TempDir::new().unwrap();
    write_hosts(temp.path(), "auth-ok");
    with_config_dir(temp.path(), || {
        let gh = fake_gh(temp.path(), 0, "{}");
        let ctx = GhContext::load_with_gh(Gh::with_executable(gh)).unwrap();

        assert_eq!(ctx.hosts().len(), 1);
        assert_eq!(ctx.default_host().unwrap().hostname, "github.com");
        assert_eq!(ctx.default_host().unwrap().user.as_deref(), Some("nubeio"));
        assert!(!ctx.default_host().unwrap().enterprise);
    });
}

#[test]
fn auth_state_uses_gh_auth_status() {
    let temp = TempDir::new().unwrap();
    write_hosts(temp.path(), "auth-ok");
    with_config_dir(temp.path(), || {
        let gh = fake_gh(temp.path(), 0, "{}");
        let ctx = GhContext::load_with_gh(Gh::with_executable(gh)).unwrap();

        assert_eq!(
            ctx.auth_state(Some("github.com")).unwrap(),
            AuthState::Authenticated
        );
    });
}

#[test]
fn auth_state_reports_unauthenticated_when_gh_status_fails() {
    let temp = TempDir::new().unwrap();
    write_hosts(temp.path(), "auth-ok");
    with_config_dir(temp.path(), || {
        let gh = fake_gh(temp.path(), 1, "not logged in");
        let ctx = GhContext::load_with_gh(Gh::with_executable(gh)).unwrap();

        assert_eq!(
            ctx.auth_state(Some("github.com")).unwrap(),
            AuthState::Unauthenticated
        );
    });
}

#[test]
fn gh_api_json_decodes_stdout() {
    let temp = TempDir::new().unwrap();
    let gh = fake_gh(temp.path(), 0, r#"{"login":"nubeio"}"#);
    let value: serde_json::Value = Gh::with_executable(gh).api_json(None, "user").unwrap();

    assert_eq!(value["login"], "nubeio");
}

#[test]
fn auth_state_reports_missing_gh() {
    let temp = TempDir::new().unwrap();
    write_hosts(temp.path(), "auth-ok");
    with_config_dir(temp.path(), || {
        let missing = temp.path().join("does-not-exist");
        let ctx = GhContext::load_with_gh(Gh::with_executable(missing)).unwrap();

        assert_eq!(ctx.auth_state(None).unwrap(), AuthState::GhMissing);
    });
}

fn write_hosts(dir: &std::path::Path, token: &str) {
    fs::write(
        dir.join("hosts.yml"),
        format!(
            r#"github.com:
    user: nubeio
    oauth_token: {token}
    git_protocol: ssh
"#
        ),
    )
    .unwrap();
}

fn with_config_dir<T>(dir: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _guard = env_lock().lock().unwrap();
    let old = std::env::var_os("GH_CONFIG_DIR");
    std::env::set_var("GH_CONFIG_DIR", dir);
    let out = f();
    if let Some(old) = old {
        std::env::set_var("GH_CONFIG_DIR", old);
    } else {
        std::env::remove_var("GH_CONFIG_DIR");
    }
    out
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(unix)]
fn fake_gh(dir: &std::path::Path, auth_status_code: i32, api_json: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join("gh");
    fs::write(
        &path,
        format!(
            r#"#!/usr/bin/env sh
if [ "$1" = "--version" ]; then
  echo "gh version test"
  exit 0
fi
if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
  echo "auth checked"
  exit {auth_status_code}
fi
if [ "$1" = "api" ]; then
  printf '%s\n' '{api_json}'
  exit 0
fi
exit 2
"#
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

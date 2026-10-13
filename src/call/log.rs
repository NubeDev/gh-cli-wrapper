//! The call log: where it lives (the application's choice), its two line shapes, and the
//! append that keeps concurrent processes from interleaving.

use std::io::Write;
use std::panic::Location;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

use super::budget::{Budget, Outcome};

/// Past this, the file becomes `<file>.1` and a new one starts. One generation is kept.
pub const ROTATE_AT: u64 = 5 * 1024 * 1024;

/// One spawn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    /// Unix seconds.
    pub at: u64,
    pub pid: u32,
    /// This process's command line, as `ghw serve --port 7777`.
    pub process: String,
    /// Its working directory — which checkout, which scratchpad.
    pub cwd: String,
    /// `crates/app/src/sprint/board_read.rs:139`.
    pub site: String,
}

/// What a call did — its own line, told apart from a [`Line`] by `outcome`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finished {
    pub at: u64,
    pub pid: u32,
    /// The site of the command it follows — the same string as that spawn's line.
    pub site: String,
    pub budget: Budget,
    pub outcome: Outcome,
    /// What GitHub charged, when it said: a GraphQL query's `rateLimit.cost` (summed over
    /// pages), or 1 for a single REST request. `None` is "GitHub did not say".
    pub cost: Option<u64>,
}

type Resolver = fn() -> Option<PathBuf>;

static RESOLVER: RwLock<Option<Resolver>> = RwLock::new(None);

/// Where the log goes, asked at every write — so an application whose state directory moves
/// with an environment variable (a test's) follows it. `None` turns logging off, which is
/// the default: a library writes no file nobody asked for.
pub fn set_log_path(resolver: Option<fn() -> Option<PathBuf>>) {
    *RESOLVER.write().unwrap_or_else(|e| e.into_inner()) = resolver;
}

/// The log's path now, if logging is on.
pub fn log_path() -> Option<PathBuf> {
    let resolver = *RESOLVER.read().unwrap_or_else(|e| e.into_inner());
    resolver.and_then(|resolve| resolve())
}

pub(super) fn spawned(site: &'static Location<'static>) {
    let Some(path) = log_path() else { return };
    let line = Line {
        at: crate::clock::now_unix(),
        pid: std::process::id(),
        process: process().to_string(),
        cwd: std::env::current_dir()
            .map(|d| d.display().to_string())
            .unwrap_or_default(),
        site: site_name(site),
    };
    let _ = append(&path, &line);
}

pub(super) fn site_name(site: &Location<'_>) -> String {
    format!("{}:{}", site.file(), site.line())
}

/// This process's command line, once: the binary's file name and its first arguments.
fn process() -> &'static str {
    static PROCESS: OnceLock<String> = OnceLock::new();
    PROCESS.get_or_init(|| {
        let mut args = std::env::args();
        let exe = args
            .next()
            .map(|a| {
                PathBuf::from(&a)
                    .file_name()
                    .map_or(a.clone(), |n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_default();
        std::iter::once(exe)
            .chain(args.take(4))
            .collect::<Vec<_>>()
            .join(" ")
    })
}

/// Append one line — a [`Line`] or a [`Finished`] — rotating first when the file is past
/// [`ROTATE_AT`].
pub fn append<T: Serialize>(path: &Path, line: &T) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if std::fs::metadata(path).is_ok_and(|m| m.len() > ROTATE_AT) {
        let _ = std::fs::rename(path, rotated(path));
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let mut text = serde_json::to_string(line).map_err(std::io::Error::other)?;
    text.push('\n');
    // One write of one short line: with O_APPEND, processes writing at once do not
    // interleave inside it.
    file.write_all(text.as_bytes())
}

/// The previous generation's path.
pub fn rotated(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".1");
    PathBuf::from(name)
}

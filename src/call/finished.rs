//! The second line: how the call just run on this thread ended, and what it cost.

use std::process::Output;

use super::budget::{Budget, Outcome};
use super::log::{append, log_path, site_name, Finished};

/// Record how the call just run on this thread ended. Never fails the call.
pub fn finished(args: &[&str], output: &Output) {
    finished_as(args, output, None);
}

/// [`finished`] for a caller that knows better than the exit code — a `304 Not Modified` is
/// `gh` exit 1 and the answer that was asked for.
pub fn finished_as(args: &[&str], output: &Output, outcome: Option<Outcome>) {
    let Some(path) = log_path() else { return };
    let Some(site) = super::command::last_site() else {
        return;
    };
    let budget = Budget::of(args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let limited =
        crate::rate_limit::is_rate_limited(&stdout) || crate::rate_limit::is_rate_limited(&stderr);
    let outcome = if let Some(outcome) = outcome {
        outcome
    } else if limited {
        Outcome::RateLimited
    } else if output.status.success() {
        Outcome::Ok
    } else {
        Outcome::Failed
    };
    let cost = match (budget, outcome) {
        (_, Outcome::RateLimited) => Some(0),
        (Budget::Graphql, _) => crate::graphql::graphql_cost(&stdout),
        (Budget::Core | Budget::Search, Outcome::Ok) if !args.contains(&"--paginate") => Some(1),
        (Budget::Free, _) => Some(0),
        _ => None,
    };
    let line = Finished {
        at: crate::clock::now_unix(),
        pid: std::process::id(),
        site: site_name(site),
        budget,
        outcome,
        cost,
    };
    let _ = append(&path, &line);
}

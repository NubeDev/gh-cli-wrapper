use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use gh_cli_wrapper::git::{GitAdd, GitCheckout, GitCommit, GitPush};
use gh_cli_wrapper::issue::{IssueClose, IssueComment, IssueCreate, IssueReopen, IssueView};
use gh_cli_wrapper::repo::RepoClone;
use gh_cli_wrapper::{Gh, GhContext};
use serde::Deserialize;
use tempfile::TempDir;

const LIVE_ENV: &str = "LIVE_GITHUB_TESTS";
const DEFAULT_REPO: &str = "NubeDev/testing";

#[test]
#[ignore = "requires LIVE_GITHUB_TESTS=1 and write access to NubeDev/testing"]
fn live_clone_branch_commit_push_issue_comment_reopen_and_close(
) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var(LIVE_ENV).as_deref() != Ok("1") {
        eprintln!("skipping live test: set {LIVE_ENV}=1");
        return Ok(());
    }

    let repo = std::env::var("LIVE_GITHUB_REPO").unwrap_or_else(|_| DEFAULT_REPO.to_string());
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let branch = format!("gh-cli-wrapper-live-{stamp}");
    let title = format!("gh-cli-wrapper live issue {stamp}");
    let body = format!("Created by gh-cli-wrapper live integration test on branch `{branch}`.");
    let comment = format!("Live comment from gh-cli-wrapper test for `{branch}`.");

    let gh = Gh::new();
    let ctx = GhContext::load()?;
    assert!(gh.is_available(), "gh must be available for live tests");
    assert_eq!(
        ctx.auth_state(Some("github.com")),
        gh_cli_wrapper::AuthState::Authenticated
    );

    let temp = TempDir::new()?;
    let checkout = temp.path().join("checkout");
    let mut clone = RepoClone::new(repo.clone());
    clone.directory = Some(checkout.display().to_string());
    clone.run(&gh)?;

    git_config(&checkout, "user.name", "gh-cli-wrapper live test")?;
    git_config(&checkout, "user.email", "gh-cli-wrapper-live@example.com")?;

    let mut git_checkout = GitCheckout::new(branch.clone());
    git_checkout.cwd = Some(checkout.clone());
    git_checkout.create = true;
    git_checkout.run()?;

    let live_dir = checkout.join(".live-tests");
    fs::create_dir_all(&live_dir)?;
    let live_file = live_dir.join(format!("{branch}.txt"));
    fs::write(
        &live_file,
        format!("repo={repo}\nbranch={branch}\nstamp={stamp}\n"),
    )?;

    let mut add = GitAdd::default();
    add.cwd = Some(checkout.clone());
    add.paths = vec![live_file.strip_prefix(&checkout)?.display().to_string()];
    add.run()?;

    let mut commit = GitCommit::new(format!("test: live gh-cli-wrapper {stamp}"));
    commit.cwd = Some(checkout.clone());
    commit.run()?;

    let mut push = GitPush::default();
    push.cwd = Some(checkout.clone());
    push.remote = Some("origin".to_string());
    push.branch = Some(branch.clone());
    push.set_upstream = true;
    push.run()?;

    let mut create = IssueCreate::default();
    create.repo = Some(repo.clone());
    create.title = Some(title.clone());
    create.body = Some(body);
    let issue_url = String::from_utf8(create.run(&gh)?.stdout)?
        .trim()
        .to_string();
    assert!(issue_url.contains(&format!("{repo}/issues/")));

    let mut issue_comment = IssueComment::new(issue_url.clone());
    issue_comment.repo = Some(repo.clone());
    issue_comment.body = Some(comment.clone());
    issue_comment.run(&gh)?;

    let mut close = IssueClose::new(issue_url.clone());
    close.repo = Some(repo.clone());
    close.comment = Some("Closing after first live assertion.".to_string());
    close.reason = Some("completed".to_string());
    close.run(&gh)?;

    let mut reopen = IssueReopen::new(issue_url.clone());
    reopen.repo = Some(repo.clone());
    reopen.comment = Some("Reopening to verify the reopen wrapper.".to_string());
    reopen.run(&gh)?;

    let mut final_close = IssueClose::new(issue_url.clone());
    final_close.repo = Some(repo.clone());
    final_close.comment = Some("Final close from gh-cli-wrapper live test.".to_string());
    final_close.reason = Some("completed".to_string());
    final_close.run(&gh)?;

    let mut view = IssueView::new(issue_url);
    view.repo = Some(repo);
    view.json_fields = vec![
        "title".to_string(),
        "state".to_string(),
        "comments".to_string(),
    ];
    let viewed: LiveIssue = view.json(&gh)?;

    assert_eq!(viewed.title, title);
    assert_eq!(viewed.state, "CLOSED");
    assert!(viewed.comments.iter().any(|item| item.body == comment));

    Ok(())
}

#[derive(Debug, Deserialize)]
struct LiveIssue {
    title: String,
    state: String,
    comments: Vec<LiveIssueComment>,
}

#[derive(Debug, Deserialize)]
struct LiveIssueComment {
    body: String,
}

fn git_config(
    cwd: &std::path::Path,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(["config", key, value])
        .output()?;
    assert!(
        output.status.success(),
        "git config failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

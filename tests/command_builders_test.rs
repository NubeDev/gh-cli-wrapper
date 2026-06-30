use std::ffi::OsString;

use gh_cli_wrapper::git::{GitAdd, GitCheckout, GitCommit, GitPush};
use gh_cli_wrapper::issue::{IssueAssign, IssueClose, IssueComment, IssueEdit};
use gh_cli_wrapper::pr::{PrMerge, PrMergeMethod};
use gh_cli_wrapper::release::ReleaseDownload;
use gh_cli_wrapper::repo::{RepoClone, RepoCreate, RepoVisibility};

#[test]
fn issue_assign_builds_issue_edit_command() {
    let mut command = IssueAssign::new("42", ["@me", "octocat"]);
    command.repo = Some("owner/repo".to_string());

    assert_eq!(
        strings(command.args()),
        [
            "issue",
            "edit",
            "42",
            "--repo",
            "owner/repo",
            "--add-assignee",
            "@me",
            "--add-assignee",
            "octocat",
        ]
    );
}

#[test]
fn issue_edit_uses_supported_project_and_milestone_flags() {
    let mut command = IssueEdit::new("12");
    command.add_projects = vec!["Roadmap".to_string()];
    command.remove_projects = vec!["Backlog".to_string()];
    command.milestone = Some("v1".to_string());

    assert_eq!(
        strings(command.args()),
        [
            "issue",
            "edit",
            "12",
            "--add-project",
            "Roadmap",
            "--remove-project",
            "Backlog",
            "--milestone",
            "v1",
        ]
    );
}

#[test]
fn issue_close_does_not_add_confirmation_flag() {
    let mut command = IssueClose::new("7");
    command.reason = Some("completed".to_string());

    assert_eq!(
        strings(command.args()),
        ["issue", "close", "7", "--reason", "completed"]
    );
}

#[test]
fn issue_comment_builds_body_command() {
    let mut command = IssueComment::new("7");
    command.repo = Some("owner/repo".to_string());
    command.body = Some("live comment".to_string());

    assert_eq!(
        strings(command.args()),
        [
            "issue",
            "comment",
            "7",
            "--repo",
            "owner/repo",
            "--body",
            "live comment",
        ]
    );
}

#[test]
fn repo_clone_allows_directory_and_passthrough_git_args() {
    let mut command = RepoClone::new("owner/repo");
    command.directory = Some("repo-copy".to_string());
    command.git_args = vec!["--depth".to_string(), "1".to_string()];

    assert_eq!(
        strings(command.args()),
        [
            "repo",
            "clone",
            "owner/repo",
            "repo-copy",
            "--",
            "--depth",
            "1",
        ]
    );
}

#[test]
fn repo_create_includes_visibility_and_common_options() {
    let mut command = RepoCreate::new("owner/new-repo");
    command.visibility = Some(RepoVisibility::Private);
    command.description = Some("wrapped".to_string());
    command.clone = true;

    assert_eq!(
        strings(command.args()),
        [
            "repo",
            "create",
            "owner/new-repo",
            "--private",
            "--description",
            "wrapped",
            "--clone",
        ]
    );
}

#[test]
fn pr_merge_selects_strategy_and_branch_cleanup() {
    let mut command = PrMerge::new("123", PrMergeMethod::Squash);
    command.delete_branch = true;

    assert_eq!(
        strings(command.args()),
        ["pr", "merge", "123", "--squash", "--delete-branch"]
    );
}

#[test]
fn release_download_builds_pattern_args() {
    let mut command = ReleaseDownload::new(Some("v1.2.3"));
    command.patterns = vec!["*.deb".to_string(), "*.rpm".to_string()];
    command.clobber = true;

    assert_eq!(
        strings(command.args()),
        [
            "release",
            "download",
            "v1.2.3",
            "--pattern",
            "*.deb",
            "--pattern",
            "*.rpm",
            "--clobber",
        ]
    );
}

#[test]
fn git_helpers_build_plain_git_commands() {
    let mut add = GitAdd::default();
    add.paths = vec!["README.md".to_string()];

    let mut checkout = GitCheckout::new("test-branch");
    checkout.create = true;

    let mut commit = GitCommit::new("ship it");
    commit.all = true;
    commit.signoff = true;

    let mut push = GitPush::default();
    push.remote = Some("origin".to_string());
    push.branch = Some("main".to_string());
    push.set_upstream = true;

    assert_eq!(add.args(), ["add", "README.md"]);
    assert_eq!(checkout.args(), ["checkout", "-b", "test-branch"]);
    assert_eq!(
        commit.args(),
        ["commit", "--message", "ship it", "--all", "--signoff"]
    );
    assert_eq!(push.args(), ["push", "--set-upstream", "origin", "main"]);
}

fn strings(args: Vec<OsString>) -> Vec<String> {
    args.into_iter()
        .map(|arg| arg.into_string().expect("test args should be UTF-8"))
        .collect()
}

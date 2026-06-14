/// Integration tests for GitCli.
/// Each test creates an isolated temp directory with a real git repo.
/// Requires: git binary on PATH, git user.name + user.email configurable.
use repodesk_git::{GitBackend, GitCli, GitError};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Helper: create a temp git repo
// ---------------------------------------------------------------------------

fn temp_dir() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "repodesk-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

fn init_repo(dir: &Path) -> GitCli {
    // Init repo
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .unwrap();
    // Set local identity so commits work without global config
    Command::new("git")
        .args(["config", "user.email", "test@repodesk.test"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Repodesk Test"])
        .current_dir(dir)
        .output()
        .unwrap();
    GitCli::new(dir)
}

fn make_initial_commit(dir: &Path) {
    fs::write(dir.join("README.md"), "# repodesk\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir)
        .output()
        .unwrap();
}

// ---------------------------------------------------------------------------
// 11. test_cli_current_branch
// ---------------------------------------------------------------------------

#[test]
fn test_cli_current_branch() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);
    let branch = git.current_branch().unwrap();
    assert_eq!(branch, "main");
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 12. test_cli_status_clean
// ---------------------------------------------------------------------------

#[test]
fn test_cli_status_clean() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);
    let entries = git.status().unwrap();
    assert!(entries.is_empty(), "expected clean status, got: {:?}", entries);
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 13. test_cli_status_dirty
// ---------------------------------------------------------------------------

#[test]
fn test_cli_status_dirty() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);
    // Add an untracked file
    fs::write(dir.join("untracked.rs"), "fn main() {}").unwrap();
    let entries = git.status().unwrap();
    assert!(!entries.is_empty(), "expected dirty status");
    let codes: Vec<&str> = entries.iter().map(|e| e.code.as_str()).collect();
    assert!(codes.contains(&"??"), "expected untracked entry, got: {:?}", codes);
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 14. test_cli_commit
// ---------------------------------------------------------------------------

#[test]
fn test_cli_commit() {
    let dir = temp_dir();
    let mut git = init_repo(&dir);
    make_initial_commit(&dir);
    // Stage a new file
    fs::write(dir.join("feature.rs"), "pub fn hello() {}").unwrap();
    Command::new("git")
        .args(["add", "feature.rs"])
        .current_dir(&dir)
        .output()
        .unwrap();
    git.commit("add feature").unwrap();
    // After commit, status should be clean
    let entries = git.status().unwrap();
    assert!(entries.is_empty(), "expected clean status after commit");
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 15. test_cli_checkout
// ---------------------------------------------------------------------------

#[test]
fn test_cli_checkout() {
    let dir = temp_dir();
    let mut git = init_repo(&dir);
    make_initial_commit(&dir);
    // Create a new branch
    Command::new("git")
        .args(["branch", "feature-x"])
        .current_dir(&dir)
        .output()
        .unwrap();
    git.checkout("feature-x").unwrap();
    let branch = git.current_branch().unwrap();
    assert_eq!(branch, "feature-x");
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 16. test_cli_not_a_repo
// ---------------------------------------------------------------------------

#[test]
fn test_cli_not_a_repo() {
    let dir = temp_dir();
    // Do NOT init a repo - plain empty directory
    let git = GitCli::new(&dir);
    let result = git.current_branch();
    assert_eq!(result, Err(GitError::NotARepo));
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// Phase 9 integration tests - diff variants
// ---------------------------------------------------------------------------

// 11. test_cli_diff_clean
#[test]
fn test_cli_diff_clean() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);

    let diff = git.diff().unwrap();
    assert!(diff.is_empty(), "expected empty diff on clean repo, got: {}", diff);
    fs::remove_dir_all(&dir).ok();
}

// 12. test_cli_diff_modified
#[test]
fn test_cli_diff_modified() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);

    // Modify the committed file.
    fs::write(dir.join("README.md"), "# modified\n").unwrap();

    let diff = git.diff().unwrap();
    assert!(!diff.is_empty(), "expected non-empty diff for modified file");
    assert!(diff.contains("modified") || diff.contains("-") || diff.contains("+"),
        "diff output unexpected: {}", diff);
    fs::remove_dir_all(&dir).ok();
}

// 13. test_cli_diff_cached
#[test]
fn test_cli_diff_cached() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);

    // Stage a new file.
    fs::write(dir.join("staged.rs"), "fn staged() {}\n").unwrap();
    Command::new("git")
        .args(["add", "staged.rs"])
        .current_dir(&dir)
        .output()
        .unwrap();

    let diff = git.diff_cached().unwrap();
    assert!(!diff.is_empty(), "expected non-empty cached diff for staged file");
    assert!(diff.contains("staged"), "expected staged.rs in cached diff: {}", diff);
    fs::remove_dir_all(&dir).ok();
}

// 14. test_cli_diff_file
#[test]
fn test_cli_diff_file() {
    let dir = temp_dir();
    let git = init_repo(&dir);
    make_initial_commit(&dir);

    // Modify README and create another file.
    fs::write(dir.join("README.md"), "# changed\n").unwrap();
    fs::write(dir.join("other.rs"), "fn other() {}\n").unwrap();

    // Diff only README.md.
    let diff = git.diff_file("README.md").unwrap();
    assert!(!diff.is_empty(), "expected diff for README.md");
    assert!(
        !diff.contains("other.rs"),
        "diff_file should not include other.rs: {}", diff
    );
    fs::remove_dir_all(&dir).ok();
}

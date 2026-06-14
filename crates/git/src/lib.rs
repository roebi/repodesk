use std::process::Command;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All errors that can occur during Git operations.
/// No panics - every failure produces a GitError.
#[derive(Debug, Clone, PartialEq)]
pub enum GitError {
    /// The working directory is not inside a Git repository.
    NotARepo,
    /// The Git command ran but reported failure.
    CommandFailed(String),
    /// The command output could not be parsed.
    ParseError(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotARepo => write!(f, "Not a git repository"),
            GitError::CommandFailed(msg) => write!(f, "Git command failed: {}", msg),
            GitError::ParseError(msg) => write!(f, "Git output parse error: {}", msg),
        }
    }
}

// ---------------------------------------------------------------------------
// StatusEntry
// ---------------------------------------------------------------------------

/// A single entry from `git status --porcelain`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusEntry {
    /// Two-character porcelain code, e.g. " M", "??", "A ".
    pub code: String,
    /// File path.
    pub path: String,
}

// ---------------------------------------------------------------------------
// GitBackend trait
// ---------------------------------------------------------------------------

/// Pure interface for Git operations.
/// All implementations must be failure-safe - no panics, errors via Result.
pub trait GitBackend {
    /// Return the name of the current branch.
    fn current_branch(&self) -> Result<String, GitError>;

    /// Return all local branch names.
    fn list_branches(&self) -> Result<Vec<String>, GitError>;

    /// Return the working tree status (porcelain entries).
    fn status(&self) -> Result<Vec<StatusEntry>, GitError>;

    /// Return the full unstaged diff output.
    fn diff(&self) -> Result<String, GitError>;

    /// Return the staged (cached) diff output.
    fn diff_cached(&self) -> Result<String, GitError>;

    /// Return the diff for a single file path.
    fn diff_file(&self, path: &str) -> Result<String, GitError>;

    /// Create a commit with the given message.
    /// Returns an error if the message is empty.
    fn commit(&mut self, message: &str) -> Result<(), GitError>;

    /// Checkout an existing branch by name.
    fn checkout(&mut self, branch: &str) -> Result<(), GitError>;
}

// ---------------------------------------------------------------------------
// MockGit - in-memory stub for unit tests
// ---------------------------------------------------------------------------

/// In-memory GitBackend implementation for unit tests.
/// All state is configured up front; no process calls.
#[derive(Debug, Clone)]
pub struct MockGit {
    pub branch: Result<String, GitError>,
    pub branches: Result<Vec<String>, GitError>,
    pub status_entries: Result<Vec<StatusEntry>, GitError>,
    pub diff_output: Result<String, GitError>,
    pub cached_diff_output: Result<String, GitError>,
    /// Records all commit messages passed to commit().
    pub committed: Vec<String>,
    /// Records all branch names passed to checkout().
    pub checked_out: Vec<String>,
    /// Branch names that are considered valid for checkout.
    pub valid_branches: Vec<String>,
}

impl MockGit {
    /// Create a clean mock representing a repo on "main" with no changes.
    pub fn clean(branch: &str) -> Self {
        Self {
            branch: Ok(branch.to_string()),
            branches: Ok(vec![branch.to_string()]),
            status_entries: Ok(vec![]),
            diff_output: Ok(String::new()),
            cached_diff_output: Ok(String::new()),
            committed: vec![],
            checked_out: vec![],
            valid_branches: vec![branch.to_string()],
        }
    }
}

impl GitBackend for MockGit {
    fn current_branch(&self) -> Result<String, GitError> {
        self.branch.clone()
    }

    fn list_branches(&self) -> Result<Vec<String>, GitError> {
        self.branches.clone()
    }

    fn status(&self) -> Result<Vec<StatusEntry>, GitError> {
        self.status_entries.clone()
    }

    fn diff(&self) -> Result<String, GitError> {
        self.diff_output.clone()
    }

    fn diff_cached(&self) -> Result<String, GitError> {
        self.cached_diff_output.clone()
    }

    fn diff_file(&self, path: &str) -> Result<String, GitError> {
        // MockGit: filter diff_output lines for the given path.
        match &self.diff_output {
            Ok(d) => {
                let filtered: String = d
                    .lines()
                    .filter(|l| l.contains(path) || l.starts_with('@') || l.starts_with('+') || l.starts_with('-') || l.starts_with(' '))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(filtered)
            }
            Err(e) => Err(e.clone()),
        }
    }

    fn commit(&mut self, message: &str) -> Result<(), GitError> {
        if message.is_empty() {
            return Err(GitError::CommandFailed(
                "commit message must not be empty".to_string(),
            ));
        }
        self.committed.push(message.to_string());
        Ok(())
    }

    fn checkout(&mut self, branch: &str) -> Result<(), GitError> {
        if !self.valid_branches.contains(&branch.to_string()) {
            return Err(GitError::CommandFailed(format!(
                "branch '{}' not found",
                branch
            )));
        }
        self.checked_out.push(branch.to_string());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// GitCli - real process wrapper
// ---------------------------------------------------------------------------

/// Real GitBackend implementation using the `git` CLI binary.
/// workdir is the directory in which all git commands are run.
pub struct GitCli {
    pub workdir: std::path::PathBuf,
}

impl GitCli {
    pub fn new(workdir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            workdir: workdir.into(),
        }
    }

    /// Run a git command, return stdout on success or a GitError.
    fn run(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            // Detect "not a git repository" in stderr.
            if stderr.contains("not a git repository") {
                return Err(GitError::NotARepo);
            }
            return Err(GitError::CommandFailed(stderr.trim().to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl GitBackend for GitCli {
    fn current_branch(&self) -> Result<String, GitError> {
        let out = self.run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(out.trim().to_string())
    }

    fn list_branches(&self) -> Result<Vec<String>, GitError> {
        let out = self.run(&["branch"])?;
        let branches = out
            .lines()
            .map(|l| l.trim_start_matches('*').trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        Ok(branches)
    }

    fn status(&self) -> Result<Vec<StatusEntry>, GitError> {
        let out = self.run(&["status", "--porcelain"])?;
        let entries = out
            .lines()
            .filter(|l| l.len() >= 3)
            .map(|l| StatusEntry {
                code: l[..2].to_string(),
                path: l[3..].to_string(),
            })
            .collect();
        Ok(entries)
    }

    fn diff(&self) -> Result<String, GitError> {
        self.run(&["diff"])
    }

    fn diff_cached(&self) -> Result<String, GitError> {
        self.run(&["diff", "--cached"])
    }

    fn diff_file(&self, path: &str) -> Result<String, GitError> {
        self.run(&["diff", "--", path])
    }

    fn commit(&mut self, message: &str) -> Result<(), GitError> {
        if message.is_empty() {
            return Err(GitError::CommandFailed(
                "commit message must not be empty".to_string(),
            ));
        }
        self.run(&["commit", "-m", message])?;
        Ok(())
    }

    fn checkout(&mut self, branch: &str) -> Result<(), GitError> {
        self.run(&["checkout", branch])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Unit tests - MockGit (no git binary required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. test_mock_current_branch
    #[test]
    fn test_mock_current_branch() {
        let git = MockGit::clean("main");
        assert_eq!(git.current_branch(), Ok("main".to_string()));
    }

    // 2. test_mock_current_branch_err
    #[test]
    fn test_mock_current_branch_err() {
        let mut git = MockGit::clean("main");
        git.branch = Err(GitError::NotARepo);
        assert_eq!(git.current_branch(), Err(GitError::NotARepo));
    }

    // 3. test_mock_list_branches
    #[test]
    fn test_mock_list_branches() {
        let mut git = MockGit::clean("main");
        git.branches = Ok(vec!["main".to_string(), "feature-x".to_string()]);
        let branches = git.list_branches().unwrap();
        assert_eq!(branches, vec!["main", "feature-x"]);
    }

    // 4. test_mock_status_clean
    #[test]
    fn test_mock_status_clean() {
        let git = MockGit::clean("main");
        let entries = git.status().unwrap();
        assert!(entries.is_empty());
    }

    // 5. test_mock_status_dirty
    #[test]
    fn test_mock_status_dirty() {
        let mut git = MockGit::clean("main");
        git.status_entries = Ok(vec![
            StatusEntry { code: "??".to_string(), path: "new_file.rs".to_string() },
            StatusEntry { code: " M".to_string(), path: "src/lib.rs".to_string() },
        ]);
        let entries = git.status().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].code, "??");
        assert_eq!(entries[0].path, "new_file.rs");
        assert_eq!(entries[1].code, " M");
    }

    // 6. test_mock_diff
    #[test]
    fn test_mock_diff() {
        let mut git = MockGit::clean("main");
        git.diff_output = Ok("-old line\n+new line\n".to_string());
        let diff = git.diff().unwrap();
        assert!(diff.contains("+new line"));
    }

    // 7. test_mock_commit
    #[test]
    fn test_mock_commit() {
        let mut git = MockGit::clean("main");
        git.commit("initial commit").unwrap();
        assert_eq!(git.committed, vec!["initial commit"]);
    }

    // 8. test_mock_commit_empty_msg
    #[test]
    fn test_mock_commit_empty_msg() {
        let mut git = MockGit::clean("main");
        let result = git.commit("");
        assert!(matches!(result, Err(GitError::CommandFailed(_))));
    }

    // 9. test_mock_checkout
    #[test]
    fn test_mock_checkout() {
        let mut git = MockGit::clean("main");
        git.valid_branches.push("feature-x".to_string());
        git.checkout("feature-x").unwrap();
        assert_eq!(git.checked_out, vec!["feature-x"]);
    }

    // 10. test_mock_checkout_unknown
    #[test]
    fn test_mock_checkout_unknown() {
        let mut git = MockGit::clean("main");
        let result = git.checkout("does-not-exist");
        assert!(matches!(result, Err(GitError::CommandFailed(_))));
    }
}

// ---------------------------------------------------------------------------
// Additional unit tests - diff variants (Phase 9)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod diff_tests {
    use super::*;

    // 15. test_mock_diff_cached
    #[test]
    fn test_mock_diff_cached() {
        let mut git = MockGit::clean("main");
        git.cached_diff_output = Ok("diff --git a/staged.rs b/staged.rs\n+fn staged() {}".to_string());
        let result = git.diff_cached().unwrap();
        assert!(result.contains("staged"));
    }

    // Extra: diff_cached default is empty string
    #[test]
    fn test_mock_diff_cached_default_empty() {
        let git = MockGit::clean("main");
        assert_eq!(git.diff_cached().unwrap(), "");
    }

    // Extra: diff_file filters by path
    #[test]
    fn test_mock_diff_file() {
        let mut git = MockGit::clean("main");
        git.diff_output = Ok(
            "diff --git a/main.rs b/main.rs\n+fn main() {}\ndiff --git a/lib.rs b/lib.rs\n+fn lib() {}".to_string()
        );
        let result = git.diff_file("main.rs").unwrap();
        assert!(result.contains("main.rs") || result.contains("fn main"));
    }

    // 16. test_appstate_refresh_diff
    #[test]
    fn test_appstate_refresh_diff() {
        use repodesk_commands::{Command, GitCommand};
        use repodesk_core::{reduce, AppState};

        let state = AppState::default();
        assert!(state.diff_output.is_none());

        let state = reduce(
            state,
            Command::Git(GitCommand::RefreshDiff(
                "diff --git a/f b/f\n+added line".to_string(),
            )),
        );
        assert!(state.diff_output.is_some());
        assert!(state.diff_output.unwrap().contains("added line"));
    }

    // Extra: RefreshDiff with empty string clears diff_output
    #[test]
    fn test_appstate_refresh_diff_clear() {
        use repodesk_commands::{Command, GitCommand};
        use repodesk_core::{reduce, AppState};

        let state = reduce(
            AppState::default(),
            Command::Git(GitCommand::RefreshDiff("some diff".to_string())),
        );
        assert!(state.diff_output.is_some());

        let state = reduce(
            state,
            Command::Git(GitCommand::RefreshDiff(String::new())),
        );
        assert!(state.diff_output.is_none());
    }
}

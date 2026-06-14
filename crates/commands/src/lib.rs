use repodesk_git::StatusEntry;

/// Editor sub-commands dispatched to the active buffer.
/// All variants are pure data - no side effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    MoveRight,
    MoveLeft,
    MoveUp,
    MoveDown,
    InsertChar(char),
    DeleteChar,
    SplitLine,
}

/// Git sub-commands for refreshing cached Git state in AppState.
/// Actual Git IO happens outside the reducer; results are pushed in
/// via these commands.
#[derive(Debug, Clone, PartialEq)]
pub enum GitCommand {
    /// Update the cached branch name.
    RefreshBranch(String),
    /// Replace the cached status entries.
    RefreshStatus(Vec<StatusEntry>),
    /// Store the latest diff output.
    RefreshDiff(String),
}

/// All commands that can be dispatched to the AppState reducer.
/// The reducer is the only entry point for state mutation.
/// Commands are pure data - no side effects, no IO.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Open a file by path into a new buffer.
    OpenFile(String),
    /// Switch the active buffer by buffer id.
    SwitchBuffer(usize),
    /// Close the buffer with the given id.
    CloseBuffer(usize),
    /// Set the status bar message.
    SetStatus(String),
    /// Set the current git branch name (legacy - kept for compatibility).
    SetGitBranch(String),
    /// Editor operation on the active buffer.
    Editor(EditorCommand),
    /// Git state refresh.
    Git(GitCommand),
    /// Command palette operation.
    Palette(PaletteCommand),
    /// Layout management operation.
    Layout(LayoutCommand),
    /// No operation - used for unknown/unhandled commands in tests.
    Noop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_clone_eq() {
        let a = Command::OpenFile("main.rs".to_string());
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_noop_variant() {
        let cmd = Command::Noop;
        assert_eq!(cmd, Command::Noop);
    }

    #[test]
    fn test_editor_command_clone_eq() {
        let a = Command::Editor(EditorCommand::InsertChar('x'));
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_git_command_clone_eq() {
        let a = Command::Git(GitCommand::RefreshBranch("main".to_string()));
        let b = a.clone();
        assert_eq!(a, b);
    }
}

/// Command palette sub-commands.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteCommand {
    /// Open the command palette.
    Open,
    /// Close the command palette and clear query.
    Close,
    /// Update the palette search query.
    SetQuery(String),
    /// Execute the result at the given index.
    SelectResult(usize),
    /// Move palette selection cursor. Positive = down, negative = up.
    MoveSelection(i32),
}

/// Layout mode toggle commands.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutCommand {
    /// Toggle between Normal and DiffOpen.
    ToggleDiff,
    /// Toggle between Normal and EditorFull.
    ToggleEditorFull,
    /// Reset to Normal layout.
    ResetLayout,
}

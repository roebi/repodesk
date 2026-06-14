use repodesk_commands::{Command, EditorCommand, GitCommand, LayoutCommand, PaletteCommand};
use repodesk_editor::Buffer;
use repodesk_git::StatusEntry;

// ---------------------------------------------------------------------------
// LayoutMode
// ---------------------------------------------------------------------------

/// Controls which panels are visible in the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    /// file tree + editor + git panel + status bar (default)
    Normal,
    /// file tree + editor + diff panel + status bar
    DiffOpen,
    /// editor only - no tree, no git panel
    EditorFull,
}

impl Default for LayoutMode {
    fn default() -> Self {
        LayoutMode::Normal
    }
}

// ---------------------------------------------------------------------------
// PaletteResult
// ---------------------------------------------------------------------------

/// A single entry in the command palette result list.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteResult {
    pub label: String,
    /// The command to dispatch when this result is selected.
    pub command: Command,
}

/// All built-in palette entries.
pub fn builtin_palette_entries() -> Vec<PaletteResult> {
    vec![
        PaletteResult { label: "Open File".to_string(),    command: Command::SetStatus(":open ".to_string()) },
        PaletteResult { label: "Save File".to_string(),    command: Command::SetStatus("save".to_string()) },
        PaletteResult { label: "Quit".to_string(),         command: Command::SetStatus("quit".to_string()) },
        PaletteResult { label: "Refresh Git".to_string(),  command: Command::Git(GitCommand::RefreshBranch(String::new())) },
        PaletteResult { label: "Toggle Diff".to_string(),  command: Command::Layout(LayoutCommand::ToggleDiff) },
        PaletteResult { label: "Editor Full".to_string(),  command: Command::Layout(LayoutCommand::ToggleEditorFull) },
    ]
}

/// Filter palette entries by query (case-insensitive substring).
pub fn filter_palette(query: &str) -> Vec<PaletteResult> {
    let q = query.to_lowercase();
    builtin_palette_entries()
        .into_iter()
        .filter(|e| e.label.to_lowercase().contains(&q))
        .collect()
}


// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Central application state.
/// All mutations go through `reduce()` - never mutated directly.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferMeta {
    pub id: usize,
    pub path: String,
    /// Editor buffer holding the file content and cursor.
    pub editor: Buffer,
}

/// Central application state.
/// All mutations go through `reduce()` - never mutated directly.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    /// All open buffers, in insertion order.
    pub buffers: Vec<BufferMeta>,
    /// Id of the currently visible buffer (None if no file open).
    pub active_buffer: Option<usize>,
    /// Path of the file in the active buffer (derived convenience field).
    pub current_file: Option<String>,
    /// Editor buffer for the active file (None if no file open).
    pub active_editor: Option<Buffer>,
    /// Status bar message shown at the bottom of the UI.
    pub status_message: String,
    /// Current git branch name (empty string = unknown/not a git repo).
    pub git_branch: String,
    /// Last known git status entries.
    pub git_status: Vec<StatusEntry>,
    /// Last known diff output (raw string from git diff).
    pub diff_output: Option<String>,
    /// Whether the command palette is open.
    pub palette_open: bool,
    /// Current palette search query.
    pub palette_query: String,
    /// Filtered palette results.
    pub palette_results: Vec<PaletteResult>,
    /// Currently selected palette result index.
    pub palette_selected: usize,
    /// Current UI layout mode.
    pub layout_mode: LayoutMode,
    /// Monotonically increasing counter for buffer ids.
    next_buffer_id: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            buffers: Vec::new(),
            active_buffer: None,
            current_file: None,
            active_editor: None,
            status_message: String::new(),
            git_branch: String::new(),
            git_status: Vec::new(),
            diff_output: None,
            palette_open: false,
            palette_query: String::new(),
            palette_results: builtin_palette_entries(),
            palette_selected: 0,
            layout_mode: LayoutMode::default(),
            next_buffer_id: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Reducer
// ---------------------------------------------------------------------------

/// Pure reducer: takes ownership of state and a command,
/// returns the next state. No side effects, no IO.
pub fn reduce(state: AppState, command: Command) -> AppState {
    match command {
        Command::OpenFile(path) => {
            let mut next = state;
            // Flush current editor state back into the buffer list before switching.
            if let Some(active_id) = next.active_buffer {
                if let Some(editor) = next.active_editor.take() {
                    if let Some(meta) = next.buffers.iter_mut().find(|b| b.id == active_id) {
                        meta.editor = editor;
                    }
                }
            }
            let id = next.next_buffer_id;
            let editor = Buffer::new();
            let meta = BufferMeta { id, path: path.clone(), editor: editor.clone() };
            next.buffers.push(meta);
            next.active_buffer = Some(id);
            next.current_file = Some(path);
            next.active_editor = Some(editor);
            next.next_buffer_id += 1;
            next
        }

        Command::SwitchBuffer(id) => {
            // Persist current editor state back into the buffer list before switching.
            let mut next = state;
            if let Some(active_id) = next.active_buffer {
                if let Some(editor) = next.active_editor.take() {
                    if let Some(meta) = next.buffers.iter_mut().find(|b| b.id == active_id) {
                        meta.editor = editor;
                    }
                }
            }
            // Switch to target buffer.
            let found = next.buffers.iter().find(|b| b.id == id).cloned();
            match found {
                Some(meta) => {
                    next.active_buffer = Some(id);
                    next.current_file = Some(meta.path.clone());
                    next.active_editor = Some(meta.editor);
                }
                None => {
                    // Restore active_editor from current active buffer (no-op switch).
                    if let Some(active_id) = next.active_buffer {
                        let restored = next
                            .buffers
                            .iter()
                            .find(|b| b.id == active_id)
                            .map(|b| b.editor.clone());
                        next.active_editor = restored;
                    }
                }
            }
            next
        }

        Command::CloseBuffer(id) => {
            let mut next = state;
            next.buffers.retain(|b| b.id != id);
            if next.active_buffer == Some(id) {
                next.active_buffer = next.buffers.last().map(|b| b.id);
                next.current_file = next
                    .buffers
                    .last()
                    .map(|b| b.path.clone());
                next.active_editor = next
                    .buffers
                    .last()
                    .map(|b| b.editor.clone());
            }
            next
        }

        Command::SetStatus(msg) => {
            let mut next = state;
            next.status_message = msg;
            next
        }

        Command::SetGitBranch(branch) => {
            let mut next = state;
            next.git_branch = branch;
            next
        }

        Command::Editor(editor_cmd) => {
            let mut next = state;
            if let Some(ref mut editor) = next.active_editor {
                match editor_cmd {
                    EditorCommand::MoveRight  => editor.move_right(),
                    EditorCommand::MoveLeft   => editor.move_left(),
                    EditorCommand::MoveUp     => editor.move_up(),
                    EditorCommand::MoveDown   => editor.move_down(),
                    EditorCommand::InsertChar(ch) => editor.insert_char(ch),
                    EditorCommand::DeleteChar => editor.delete_char(),
                    EditorCommand::SplitLine  => editor.split_line(),
                }
            }
            // If no active_editor: noop (failure-safe).
            next
        }

        Command::Git(git_cmd) => {
            let mut next = state;
            match git_cmd {
                GitCommand::RefreshBranch(branch) => {
                    next.git_branch = branch;
                }
                GitCommand::RefreshStatus(entries) => {
                    next.git_status = entries;
                }
                GitCommand::RefreshDiff(diff) => {
                    next.diff_output = if diff.is_empty() { None } else { Some(diff) };
                }
            }
            next
        }

        Command::Palette(palette_cmd) => {
            let mut next = state;
            match palette_cmd {
                PaletteCommand::Open => {
                    next.palette_open = true;
                    next.palette_query = String::new();
                    next.palette_results = builtin_palette_entries();
                    next.palette_selected = 0;
                }
                PaletteCommand::Close => {
                    next.palette_open = false;
                    next.palette_query = String::new();
                    next.palette_results = builtin_palette_entries();
                    next.palette_selected = 0;
                }
                PaletteCommand::SetQuery(q) => {
                    next.palette_results = filter_palette(&q);
                    next.palette_selected = 0;
                    next.palette_query = q;
                }
                PaletteCommand::SelectResult(idx) => {
                    // Close palette; actual command dispatch happens in ui/app.rs
                    if idx < next.palette_results.len() {
                        next.palette_selected = idx;
                    }
                    next.palette_open = false;
                }
                PaletteCommand::MoveSelection(delta) => {
                    let len = next.palette_results.len();
                    if len > 0 {
                        let current = next.palette_selected as i64;
                        let new_idx = (current + delta as i64)
                            .max(0)
                            .min((len - 1) as i64) as usize;
                        next.palette_selected = new_idx;
                    }
                }
            }
            next
        }

        Command::Layout(layout_cmd) => {
            let mut next = state;
            match layout_cmd {
                LayoutCommand::ToggleDiff => {
                    next.layout_mode = match next.layout_mode {
                        LayoutMode::DiffOpen => LayoutMode::Normal,
                        _ => LayoutMode::DiffOpen,
                    };
                }
                LayoutCommand::ToggleEditorFull => {
                    next.layout_mode = match next.layout_mode {
                        LayoutMode::EditorFull => LayoutMode::Normal,
                        _ => LayoutMode::EditorFull,
                    };
                }
                LayoutCommand::ResetLayout => {
                    next.layout_mode = LayoutMode::Normal;
                }
            }
            next
        }

        Command::Noop => state,
    }
}

// ---------------------------------------------------------------------------
// Tests - TDD Phase 4
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use repodesk_commands::{Command, EditorCommand, GitCommand};
    use repodesk_editor::Cursor;
    use repodesk_git::StatusEntry;

    // helpers
    fn open(state: AppState, path: &str) -> AppState {
        reduce(state, Command::OpenFile(path.to_string()))
    }

    // 1. test_open_file_creates_editor_buffer
    #[test]
    fn test_open_file_creates_editor_buffer() {
        let state = open(AppState::default(), "src/main.rs");
        assert!(state.active_editor.is_some());
    }

    // 2. test_open_file_buffer_is_empty
    #[test]
    fn test_open_file_buffer_is_empty() {
        let state = open(AppState::default(), "src/main.rs");
        let editor = state.active_editor.unwrap();
        assert_eq!(editor.lines(), &[""]);
        assert_eq!(editor.cursor, Cursor { row: 0, col: 0 });
    }

    // 3. test_editor_insert_char
    #[test]
    fn test_editor_insert_char() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('x')));
        let editor = state.active_editor.unwrap();
        assert_eq!(editor.lines()[0], "x");
        assert_eq!(editor.cursor.col, 1);
    }

    // 4. test_editor_move_right
    #[test]
    fn test_editor_move_right() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('h')));
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('i')));
        // cursor is at col 2 (end); move left first then right
        let state = reduce(state, Command::Editor(EditorCommand::MoveLeft));
        let state = reduce(state, Command::Editor(EditorCommand::MoveRight));
        assert_eq!(state.active_editor.unwrap().cursor.col, 2);
    }

    // 5. test_editor_move_left
    #[test]
    fn test_editor_move_left() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('h')));
        let state = reduce(state, Command::Editor(EditorCommand::MoveLeft));
        assert_eq!(state.active_editor.unwrap().cursor.col, 0);
    }

    // 6. test_editor_move_up
    #[test]
    fn test_editor_move_up() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('a')));
        let state = reduce(state, Command::Editor(EditorCommand::SplitLine));
        // cursor is now row=1
        assert_eq!(state.active_editor.as_ref().unwrap().cursor.row, 1);
        let state = reduce(state, Command::Editor(EditorCommand::MoveUp));
        assert_eq!(state.active_editor.unwrap().cursor.row, 0);
    }

    // 7. test_editor_move_down
    #[test]
    fn test_editor_move_down() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('a')));
        let state = reduce(state, Command::Editor(EditorCommand::SplitLine));
        let state = reduce(state, Command::Editor(EditorCommand::MoveUp));
        // cursor is at row=0, move down
        let state = reduce(state, Command::Editor(EditorCommand::MoveDown));
        assert_eq!(state.active_editor.unwrap().cursor.row, 1);
    }

    // 8. test_editor_delete_char
    #[test]
    fn test_editor_delete_char() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('x')));
        let state = reduce(state, Command::Editor(EditorCommand::DeleteChar));
        let editor = state.active_editor.unwrap();
        assert_eq!(editor.lines()[0], "");
        assert_eq!(editor.cursor.col, 0);
    }

    // 9. test_editor_split_line
    #[test]
    fn test_editor_split_line() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('a')));
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('b')));
        let state = reduce(state, Command::Editor(EditorCommand::MoveLeft));
        // cursor at col=1, split
        let state = reduce(state, Command::Editor(EditorCommand::SplitLine));
        let editor = state.active_editor.unwrap();
        assert_eq!(editor.lines(), &["a", "b"]);
        assert_eq!(editor.cursor, Cursor { row: 1, col: 0 });
    }

    // 10. test_editor_cmd_no_active_buffer
    #[test]
    fn test_editor_cmd_no_active_buffer() {
        // No file open - EditorCommand must be a noop, no panic
        let state = AppState::default();
        assert!(state.active_editor.is_none());
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('x')));
        assert!(state.active_editor.is_none());
    }

    // 11. test_git_refresh_branch
    #[test]
    fn test_git_refresh_branch() {
        let state = AppState::default();
        let state = reduce(state, Command::Git(GitCommand::RefreshBranch("develop".to_string())));
        assert_eq!(state.git_branch, "develop");
    }

    // 12. test_git_refresh_status_clean
    #[test]
    fn test_git_refresh_status_clean() {
        let state = AppState::default();
        let state = reduce(state, Command::Git(GitCommand::RefreshStatus(vec![])));
        assert!(state.git_status.is_empty());
    }

    // 13. test_git_refresh_status_dirty
    #[test]
    fn test_git_refresh_status_dirty() {
        let state = AppState::default();
        let entries = vec![
            StatusEntry { code: "??".to_string(), path: "new.rs".to_string() },
            StatusEntry { code: " M".to_string(), path: "lib.rs".to_string() },
        ];
        let state = reduce(state, Command::Git(GitCommand::RefreshStatus(entries.clone())));
        assert_eq!(state.git_status.len(), 2);
        assert_eq!(state.git_status[0].code, "??");
        assert_eq!(state.git_status[1].path, "lib.rs");
    }

    // 14. test_switch_buffer_swaps_editor
    #[test]
    fn test_switch_buffer_swaps_editor() {
        let state = AppState::default();
        let state = open(state, "a.rs"); // id=0
        // type into a.rs
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('A')));
        let state = open(state, "b.rs"); // id=1
        // type into b.rs
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('B')));
        // switch back to a.rs
        let state = reduce(state, Command::SwitchBuffer(0));
        let editor = state.active_editor.unwrap();
        assert_eq!(editor.lines()[0], "A");
    }

    // 15. test_close_buffer_clears_editor
    #[test]
    fn test_close_buffer_clears_editor() {
        let state = open(AppState::default(), "only.rs"); // id=0
        let state = reduce(state, Command::CloseBuffer(0));
        assert!(state.active_editor.is_none());
        assert!(state.buffers.is_empty());
    }

    // 16. test_reducer_pure_with_editor
    #[test]
    fn test_reducer_pure_with_editor() {
        let a = open(AppState::default(), "main.rs");
        let b = open(AppState::default(), "main.rs");
        let cmd_a = Command::Editor(EditorCommand::InsertChar('z'));
        let cmd_b = Command::Editor(EditorCommand::InsertChar('z'));
        assert_eq!(reduce(a, cmd_a), reduce(b, cmd_b));
    }

    // --- Phase 1 regression tests (keep all green) ---

    #[test]
    fn test_initial_state() {
        let state = AppState::default();
        assert!(state.buffers.is_empty());
        assert_eq!(state.active_buffer, None);
        assert_eq!(state.current_file, None);
        assert_eq!(state.active_editor, None);
        assert_eq!(state.status_message, "");
        assert_eq!(state.git_branch, "");
        assert!(state.git_status.is_empty());
    }

    #[test]
    fn test_reducer_noop() {
        let before = AppState::default();
        let after = reduce(before.clone(), Command::Noop);
        assert_eq!(before, after);
    }

    #[test]
    fn test_set_status_message() {
        let state = reduce(AppState::default(), Command::SetStatus("saved".to_string()));
        assert_eq!(state.status_message, "saved");
    }

    #[test]
    fn test_set_git_branch_legacy() {
        let state = reduce(AppState::default(), Command::SetGitBranch("main".to_string()));
        assert_eq!(state.git_branch, "main");
    }
}

// ---------------------------------------------------------------------------
// Phase 10 tests - palette + layout
// ---------------------------------------------------------------------------

#[cfg(test)]
mod phase10_tests {
    use super::*;
    use repodesk_commands::{Command, LayoutCommand, PaletteCommand};

    // 1. test_palette_opens
    #[test]
    fn test_palette_opens() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        assert!(state.palette_open);
    }

    // 2. test_palette_closes
    #[test]
    fn test_palette_closes() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::Close));
        assert!(!state.palette_open);
    }

    // 3. test_palette_set_query
    #[test]
    fn test_palette_set_query() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::SetQuery("op".to_string())));
        assert_eq!(state.palette_query, "op");
    }

    // 4. test_palette_results_filtered
    #[test]
    fn test_palette_results_filtered() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::SetQuery("op".to_string())));
        assert!(!state.palette_results.is_empty());
        assert!(state.palette_results.iter().any(|r| r.label.to_lowercase().contains("op")),
            "expected 'op' match in results: {:?}", state.palette_results.iter().map(|r| &r.label).collect::<Vec<_>>());
    }

    // 5. test_palette_results_empty_query
    #[test]
    fn test_palette_results_empty_query() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::SetQuery(String::new())));
        let total = builtin_palette_entries().len();
        assert_eq!(state.palette_results.len(), total,
            "empty query should return all {} entries", total);
    }

    // 6. test_palette_move_selection
    #[test]
    fn test_palette_move_selection() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        assert_eq!(state.palette_selected, 0);
        let state = reduce(state, Command::Palette(PaletteCommand::MoveSelection(1)));
        assert_eq!(state.palette_selected, 1);
    }

    // 7. test_palette_move_clamps_top
    #[test]
    fn test_palette_move_clamps_top() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::MoveSelection(-1)));
        assert_eq!(state.palette_selected, 0);
    }

    // 8. test_palette_move_clamps_bottom
    #[test]
    fn test_palette_move_clamps_bottom() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let last = state.palette_results.len() - 1;
        // Move far past end
        let state = reduce(state, Command::Palette(PaletteCommand::MoveSelection(999)));
        assert_eq!(state.palette_selected, last);
    }

    // 9. test_palette_close_clears_query
    #[test]
    fn test_palette_close_clears_query() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::SetQuery("git".to_string())));
        assert_eq!(state.palette_query, "git");
        let state = reduce(state, Command::Palette(PaletteCommand::Close));
        assert!(state.palette_query.is_empty());
        assert!(!state.palette_open);
    }

    // 10. test_layout_normal_has_git_panel
    #[test]
    fn test_layout_normal_has_git_panel() {
        let state = AppState::default();
        assert_eq!(state.layout_mode, LayoutMode::Normal);
    }

    // 11. test_layout_diff_open
    #[test]
    fn test_layout_diff_open() {
        let state = reduce(AppState::default(), Command::Layout(LayoutCommand::ToggleDiff));
        assert_eq!(state.layout_mode, LayoutMode::DiffOpen);
    }

    // 12. test_layout_editor_full
    #[test]
    fn test_layout_editor_full() {
        let state = reduce(AppState::default(), Command::Layout(LayoutCommand::ToggleEditorFull));
        assert_eq!(state.layout_mode, LayoutMode::EditorFull);
    }

    // 13. test_toggle_diff_mode
    #[test]
    fn test_toggle_diff_mode() {
        let state = reduce(AppState::default(), Command::Layout(LayoutCommand::ToggleDiff));
        assert_eq!(state.layout_mode, LayoutMode::DiffOpen);
        // Toggle back to Normal
        let state = reduce(state, Command::Layout(LayoutCommand::ToggleDiff));
        assert_eq!(state.layout_mode, LayoutMode::Normal);
    }

    // Extra: ResetLayout always goes to Normal
    #[test]
    fn test_reset_layout() {
        let state = reduce(AppState::default(), Command::Layout(LayoutCommand::ToggleEditorFull));
        let state = reduce(state, Command::Layout(LayoutCommand::ResetLayout));
        assert_eq!(state.layout_mode, LayoutMode::Normal);
    }

    // Extra: SelectResult closes palette
    #[test]
    fn test_palette_select_closes() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        assert!(state.palette_open);
        let state = reduce(state, Command::Palette(PaletteCommand::SelectResult(0)));
        assert!(!state.palette_open);
        assert_eq!(state.palette_selected, 0);
    }
}

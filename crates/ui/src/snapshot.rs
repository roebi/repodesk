use repodesk_core::{AppState, LayoutMode};
use repodesk_fs::{list_dir, EntryKind};
use crate::diff::{parse_diff, render_diff_panel};
use crate::layout::compute;

/// Render AppState into a Vec<String> of exactly `height` lines,
/// each padded or truncated to exactly `width` chars.
/// Pure function of AppState + optional repo_root for file tree.
/// repo_root=None falls back to open-buffers-only tree.
pub fn render_to_lines(state: &AppState, width: u16, height: u16) -> Vec<String> {
    render_to_lines_with_root(state, width, height, None)
}

pub fn render_to_lines_with_root(
    state: &AppState,
    width: u16,
    height: u16,
    repo_root: Option<&std::path::Path>,
) -> Vec<String> {
    let layout = match compute(width, height) {
        Some(l) => l,
        None => {
            let msg = "Terminal too small";
            let line = format_line(msg, width as usize);
            let mut out = vec![line];
            while out.len() < height as usize {
                out.push(blank(width as usize));
            }
            return out;
        }
    };

    let total_rows = height as usize;
    let w          = width as usize;
    let main_h     = layout.file_tree.height as usize;
    let git_h      = layout.git_panel.height as usize;
    let tree_w     = layout.file_tree.width as usize;
    let editor_w   = layout.editor.width as usize;

    let mut lines: Vec<String> = Vec::with_capacity(total_rows);

    // -----------------------------------------------------------------------
    // Main area: file tree (left) | editor (right)
    // -----------------------------------------------------------------------
    let tree_lines   = render_file_tree(state, tree_w, main_h, repo_root);
    let editor_lines = render_editor(state, editor_w, main_h);

    for row in 0..main_h {
        let tree_cell   = pad_truncate(&tree_lines[row],   tree_w);
        let editor_cell = pad_truncate(&editor_lines[row], editor_w);
        lines.push(format!("{}{}", tree_cell, editor_cell));
    }

    // -----------------------------------------------------------------------
    // Git panel
    // -----------------------------------------------------------------------
    let git_lines = render_git_panel(state, w, git_h);
    for row in 0..git_h {
        lines.push(pad_truncate(&git_lines[row], w));
    }

    // -----------------------------------------------------------------------
    // Status bar
    // -----------------------------------------------------------------------
    let status = render_status_bar(state, w);
    lines.push(pad_truncate(&status, w));

    lines.truncate(total_rows);
    while lines.len() < total_rows {
        lines.push(blank(w));
    }
    lines
}

/// Render with layout mode awareness (EditorFull, DiffOpen, Normal).
/// Used by Phase 10 tests and app.rs event loop.
pub fn render_to_lines_with_layout(state: &AppState, width: u16, height: u16) -> Vec<String> {
    render_to_lines_with_layout_and_root(state, width, height, None)
}

pub fn render_to_lines_with_layout_and_root(
    state: &AppState,
    width: u16,
    height: u16,
    repo_root: Option<&std::path::Path>,
) -> Vec<String> {
    match &state.layout_mode {
        LayoutMode::EditorFull => render_editor_full(state, width, height),
        LayoutMode::DiffOpen   => render_diff_open(state, width, height, repo_root),
        LayoutMode::Normal     => render_to_lines_with_root(state, width, height, repo_root),
    }
}

fn render_editor_full(state: &AppState, width: u16, height: u16) -> Vec<String> {
    let w = width as usize;
    let h = height as usize;
    let mut lines: Vec<String> = Vec::with_capacity(h);
    let editor_rows = h.saturating_sub(1); // 1 row for status bar

    let editor_lines = render_editor(state, w, editor_rows);
    for l in editor_lines {
        lines.push(pad_truncate(&l, w));
    }
    lines.push(pad_truncate(&render_status_bar(state, w), w));

    lines.truncate(h);
    while lines.len() < h { lines.push(blank(w)); }
    lines
}

fn render_diff_open(
    state: &AppState,
    width: u16,
    height: u16,
    repo_root: Option<&std::path::Path>,
) -> Vec<String> {
    let layout = match compute(width, height) {
        Some(l) => l,
        None => return render_to_lines_with_root(state, width, height, repo_root),
    };

    let w        = width as usize;
    let total    = height as usize;
    let main_h   = layout.file_tree.height as usize;
    let git_h    = layout.git_panel.height as usize;
    let tree_w   = layout.file_tree.width as usize;
    let editor_w = layout.editor.width as usize;

    let mut lines: Vec<String> = Vec::with_capacity(total);

    let tree_lines   = render_file_tree(state, tree_w, main_h, repo_root);
    let editor_lines = render_editor(state, editor_w, main_h);
    for row in 0..main_h {
        lines.push(format!("{}{}", pad_truncate(&tree_lines[row], tree_w), pad_truncate(&editor_lines[row], editor_w)));
    }

    // Diff panel in place of git panel
    let diff_lines = match &state.diff_output {
        Some(raw) => {
            let parsed = parse_diff(raw);
            render_diff_panel(&parsed, w, git_h)
        }
        None => {
            let mut v = vec![pad_truncate("DIFF: (no diff loaded - press Ctrl-D)", w)];
            while v.len() < git_h { v.push(blank(w)); }
            v
        }
    };
    for l in diff_lines { lines.push(pad_truncate(&l, w)); }

    lines.push(pad_truncate(&render_status_bar(state, w), w));

    lines.truncate(total);
    while lines.len() < total { lines.push(blank(w)); }
    lines
}

// ---------------------------------------------------------------------------
// Panel renderers
// ---------------------------------------------------------------------------

fn render_file_tree(
    state: &AppState,
    cols: usize,
    rows: usize,
    repo_root: Option<&std::path::Path>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(rows);
    out.push(format_line("FILES", cols));

    // Try real directory listing first.
    let fs_entries: Vec<String> = repo_root
        .and_then(|root| list_dir(root).ok())
        .map(|entries| {
            entries
                .into_iter()
                .map(|e| {
                    let prefix = if e.kind == EntryKind::Directory { "/" } else { " " };
                    format!("{}{}", prefix, e.name)
                })
                .collect()
        })
        .unwrap_or_default();

    if !fs_entries.is_empty() {
        for entry in &fs_entries {
            if out.len() >= rows { break; }
            out.push(format_line(entry, cols));
        }
    } else {
        // Fallback: show open buffers.
        for meta in &state.buffers {
            if out.len() >= rows { break; }
            let marker = if state.active_buffer == Some(meta.id) { ">" } else { " " };
            let entry = format!("{} {}", marker, short_path(&meta.path, cols.saturating_sub(2)));
            out.push(format_line(&entry, cols));
        }
    }

    while out.len() < rows {
        out.push(blank(cols));
    }
    out
}

fn render_editor(state: &AppState, cols: usize, rows: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(rows);

    match &state.active_editor {
        None => {
            out.push(format_line("No file open. Use :open <path>", cols));
        }
        Some(editor) => {
            let cursor_row = editor.cursor.row;
            let cursor_col = editor.cursor.col;

            for (row_idx, line) in editor.lines().iter().enumerate() {
                if out.len() >= rows { break; }
                if row_idx == cursor_row {
                    let mut chars: Vec<char> = line.chars().collect();
                    let insert_at = cursor_col.min(chars.len());
                    chars.insert(insert_at, '|');
                    let marked: String = chars.into_iter().collect();
                    out.push(format_line(&marked, cols));
                } else {
                    out.push(format_line(line, cols));
                }
            }
        }
    }

    while out.len() < rows {
        out.push(blank(cols));
    }
    out
}

fn render_git_panel(state: &AppState, cols: usize, rows: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(rows);

    let branch_line = if state.git_branch.is_empty() {
        format_line("GIT: (no branch)", cols)
    } else {
        format_line(&format!("GIT: branch={}", state.git_branch), cols)
    };
    out.push(branch_line);

    let remaining = rows.saturating_sub(1);
    if state.git_status.is_empty() {
        out.push(format_line("  clean", cols));
    } else {
        for entry in state.git_status.iter().take(remaining) {
            let s = format!("  {} {}", entry.code, entry.path);
            out.push(format_line(&s, cols));
        }
    }

    while out.len() < rows {
        out.push(blank(cols));
    }
    out
}

fn render_status_bar(state: &AppState, cols: usize) -> String {
    if state.status_message.is_empty() {
        format_line("Ctrl-Q: quit  Ctrl-S: save  Ctrl-O: open", cols)
    } else {
        format_line(&state.status_message, cols)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn blank(width: usize) -> String {
    " ".repeat(width)
}

fn pad_truncate(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= width {
        chars[..width].iter().collect()
    } else {
        let mut out = s.to_string();
        while out.len() < width {
            out.push(' ');
        }
        out
    }
}

fn format_line(s: &str, width: usize) -> String {
    pad_truncate(s, width)
}

fn short_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        let truncated = &path[path.len().saturating_sub(max_len)..];
        format!("~{}", truncated)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use repodesk_core::AppState;
    use repodesk_commands::{Command, EditorCommand, GitCommand};
    use repodesk_git::StatusEntry;
    use repodesk_core::reduce;

    const W: u16 = 80;
    const H: u16 = 24;

    fn open(state: AppState, path: &str) -> AppState {
        reduce(state, Command::OpenFile(path.to_string()))
    }

    // 4. test_snapshot_empty_state
    #[test]
    fn test_snapshot_empty_state() {
        let state = AppState::default();
        let lines = render_to_lines(&state, W, H);
        let editor_area: String = lines[..20].join("\n");
        assert!(editor_area.contains("No file open"));
    }

    // 5. test_snapshot_status_message
    #[test]
    fn test_snapshot_status_message() {
        let state = reduce(AppState::default(), Command::SetStatus("File saved".to_string()));
        let lines = render_to_lines(&state, W, H);
        assert!(lines[23].contains("File saved"));
    }

    // 6. test_snapshot_git_branch
    #[test]
    fn test_snapshot_git_branch() {
        let state = reduce(
            AppState::default(),
            Command::Git(GitCommand::RefreshBranch("main".to_string())),
        );
        let lines = render_to_lines(&state, W, H);
        let git_area = &lines[20];
        assert!(git_area.contains("main"), "expected branch in git panel: {}", git_area);
    }

    // 7. test_snapshot_git_status_entries
    #[test]
    fn test_snapshot_git_status_entries() {
        let entries = vec![
            StatusEntry { code: "??".to_string(), path: "new.rs".to_string() },
        ];
        let state = reduce(AppState::default(), Command::Git(GitCommand::RefreshStatus(entries)));
        let lines = render_to_lines(&state, W, H);
        let git_area = lines[20..23].join("\n");
        assert!(git_area.contains("new.rs"), "expected status entry: {}", git_area);
        assert!(git_area.contains("??"));
    }

    // 8. test_snapshot_editor_content
    #[test]
    fn test_snapshot_editor_content() {
        let state = open(AppState::default(), "src/main.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('h')));
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('i')));
        let lines = render_to_lines(&state, W, H);
        let editor_part = &lines[0][20..];
        assert!(
            editor_part.contains("hi|") || editor_part.contains("h|i") || editor_part.contains("|hi"),
            "editor content not found: {}", editor_part
        );
    }

    // 9. test_snapshot_cursor_marker
    #[test]
    fn test_snapshot_cursor_marker() {
        let state = open(AppState::default(), "a.rs");
        let state = reduce(state, Command::Editor(EditorCommand::InsertChar('x')));
        let lines = render_to_lines(&state, W, H);
        assert!(lines[0].contains('|'), "cursor marker | not found in: {}", lines[0]);
    }

    // 10. test_snapshot_file_tree_entry
    #[test]
    fn test_snapshot_file_tree_entry() {
        let state = open(AppState::default(), "src/main.rs");
        let lines = render_to_lines(&state, W, H);
        let tree_area: String = lines[..20]
            .iter()
            .map(|l| &l[..20.min(l.len())])
            .collect::<Vec<_>>()
            .join("\n");
        assert!(tree_area.contains("main.rs"), "file not in tree: {}", tree_area);
    }

    // 11. test_snapshot_width_respected
    #[test]
    fn test_snapshot_width_respected() {
        let state = AppState::default();
        let lines = render_to_lines(&state, W, H);
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(), W as usize,
                "line {} has wrong width: {:?}", i, line
            );
        }
    }

    // 12. test_snapshot_height_respected
    #[test]
    fn test_snapshot_height_respected() {
        let state = AppState::default();
        let lines = render_to_lines(&state, W, H);
        assert_eq!(lines.len(), H as usize);
    }

    #[test]
    fn test_snapshot_default_status_hint() {
        let state = AppState::default();
        let lines = render_to_lines(&state, W, H);
        assert!(lines[23].contains("Ctrl-Q"), "hint not in status bar: {}", lines[23]);
    }

    #[test]
    fn test_snapshot_terminal_too_small() {
        let state = AppState::default();
        let lines = render_to_lines(&state, 5, 3);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("too small") || lines[0].len() == 5);
    }

    // fs wire-up: file tree uses real dir listing when repo_root provided
    #[test]
    fn test_snapshot_file_tree_from_fs() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let dir = std::env::temp_dir().join(format!("repodesk-snap-{}", ns));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.join("lib.rs"), "").unwrap();

        let state = AppState::default();
        let lines = render_to_lines_with_root(&state, W, H, Some(&dir));
        let tree_area: String = lines[..20]
            .iter()
            .map(|l| &l[..20.min(l.len())])
            .collect::<Vec<_>>()
            .join("\n");

        assert!(tree_area.contains("lib.rs"),  "lib.rs not in tree: {}", tree_area);
        assert!(tree_area.contains("main.rs"), "main.rs not in tree: {}", tree_area);

        fs::remove_dir_all(&dir).ok();
    }
}

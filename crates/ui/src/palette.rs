use repodesk_core::{AppState, LayoutMode};

// ---------------------------------------------------------------------------
// Palette overlay renderer
// ---------------------------------------------------------------------------

/// Render the command palette as a centred overlay on top of existing lines.
/// Mutates the passed `lines` vec in place.
/// Palette box: width=50, height=results+3 (border + query + results).
pub fn render_palette_overlay(lines: &mut Vec<String>, state: &AppState, total_width: usize) {
    if !state.palette_open {
        return;
    }

    let box_width: usize = 52.min(total_width);
    let box_left = total_width.saturating_sub(box_width) / 2;

    // Top border
    let top = format!(
        "{}+{}+",
        " ".repeat(box_left),
        "-".repeat(box_width - 2)
    );

    // Query line
    let query_display = format!("> {}", state.palette_query);
    let query_line = format!(
        "{}|{}|",
        " ".repeat(box_left),
        pad_truncate(&query_display, box_width - 2)
    );

    // Separator
    let sep = format!(
        "{}|{}|",
        " ".repeat(box_left),
        "-".repeat(box_width - 2)
    );

    // Result lines (up to 8)
    let result_lines: Vec<String> = state
        .palette_results
        .iter()
        .enumerate()
        .take(8)
        .map(|(i, r)| {
            let marker = if i == state.palette_selected { ">" } else { " " };
            let label = format!("{} {}", marker, r.label);
            format!(
                "{}|{}|",
                " ".repeat(box_left),
                pad_truncate(&label, box_width - 2)
            )
        })
        .collect();

    // Bottom border
    let bottom = format!(
        "{}+{}+",
        " ".repeat(box_left),
        "-".repeat(box_width - 2)
    );

    // Overlay onto lines starting at row 2
    let mut row = 2usize;
    for overlay_line in std::iter::once(&top)
        .chain(std::iter::once(&query_line))
        .chain(std::iter::once(&sep))
        .chain(result_lines.iter())
        .chain(std::iter::once(&bottom))
    {
        if row >= lines.len() {
            break;
        }
        // Replace the section of the line covered by the box.
        let orig: Vec<char> = lines[row].chars().collect();
        let over: Vec<char> = overlay_line.chars().collect();
        let mut merged: Vec<char> = orig.clone();
        for (ci, &oc) in over.iter().enumerate() {
            if ci < merged.len() {
                merged[ci] = oc;
            }
        }
        lines[row] = merged.into_iter().collect();
        row += 1;
    }
}

// ---------------------------------------------------------------------------
// Layout-aware render helper
// ---------------------------------------------------------------------------

/// Returns true if the git/diff panel should show diff content
/// (DiffOpen mode) rather than git status.
pub fn layout_shows_diff(state: &AppState) -> bool {
    state.layout_mode == LayoutMode::DiffOpen
}

/// Returns true if the file tree panel should be hidden (EditorFull mode).
pub fn layout_hides_tree(state: &AppState) -> bool {
    state.layout_mode == LayoutMode::EditorFull
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests (14-18)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use repodesk_commands::{Command, LayoutCommand, PaletteCommand};
    use repodesk_core::{reduce, AppState};
    use crate::snapshot::render_to_lines;

    const W: u16 = 80;
    const H: u16 = 24;

    // 14. test_snapshot_palette_overlay
    #[test]
    fn test_snapshot_palette_overlay() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let mut lines = render_to_lines(&state, W, H);
        render_palette_overlay(&mut lines, &state, W as usize);
        let combined = lines.join("\n");
        assert!(combined.contains('+') || combined.contains('|'),
            "palette border not found in snapshot");
    }

    // 15. test_snapshot_palette_query
    #[test]
    fn test_snapshot_palette_query() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let state = reduce(state, Command::Palette(PaletteCommand::SetQuery("git".to_string())));
        let mut lines = render_to_lines(&state, W, H);
        render_palette_overlay(&mut lines, &state, W as usize);
        let combined = lines.join("\n");
        assert!(combined.contains("git"), "query 'git' not visible in palette: {}", combined);
    }

    // 16. test_snapshot_palette_results
    #[test]
    fn test_snapshot_palette_results() {
        let state = reduce(AppState::default(), Command::Palette(PaletteCommand::Open));
        let mut lines = render_to_lines(&state, W, H);
        render_palette_overlay(&mut lines, &state, W as usize);
        let combined = lines.join("\n");
        assert!(combined.contains("Open File") || combined.contains("Save"),
            "palette results not visible: {}", combined);
    }

    // 17. test_snapshot_layout_editor_full
    #[test]
    fn test_snapshot_layout_editor_full() {
        use crate::snapshot::render_to_lines_with_layout;
        let state = reduce(AppState::default(), Command::Layout(LayoutCommand::ToggleEditorFull));
        let lines = render_to_lines_with_layout(&state, W, H);
        assert_eq!(lines.len(), H as usize);
        // In EditorFull, no "FILES" header in tree area (tree hidden).
        let combined = lines.join("\n");
        assert!(!combined[..20].contains("FILES"),
            "FILES header should be absent in EditorFull: {}", &combined[..80]);
    }

    // 18. test_snapshot_layout_diff_open
    #[test]
    fn test_snapshot_layout_diff_open() {
        use crate::snapshot::render_to_lines_with_layout;
        use repodesk_commands::GitCommand;

        let state = reduce(AppState::default(), Command::Git(GitCommand::RefreshDiff(
            "diff --git a/f b/f\n+added line".to_string(),
        )));
        let state = reduce(state, Command::Layout(LayoutCommand::ToggleDiff));
        let lines = render_to_lines_with_layout(&state, W, H);
        let git_area = lines[20..23].join("\n");
        assert!(git_area.contains('+') || git_area.contains("added") || git_area.contains("diff"),
            "diff content not in git panel area for DiffOpen: {}", git_area);
    }

    // Extra: palette not rendered when closed
    #[test]
    fn test_palette_not_rendered_when_closed() {
        let state = AppState::default();
        assert!(!state.palette_open);
        let mut lines = render_to_lines(&state, W, H);
        let before = lines.join("\n");
        render_palette_overlay(&mut lines, &state, W as usize);
        let after = lines.join("\n");
        assert_eq!(before, after, "closed palette should not change lines");
    }

    // Extra: layout_shows_diff correct
    #[test]
    fn test_layout_shows_diff_flag() {
        let normal = AppState::default();
        assert!(!layout_shows_diff(&normal));
        let diff_open = reduce(normal, Command::Layout(LayoutCommand::ToggleDiff));
        assert!(layout_shows_diff(&diff_open));
    }
}

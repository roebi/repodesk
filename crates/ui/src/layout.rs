/// A rectangular region in the terminal (zero-based, inclusive top-left).
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }
}

/// All panel rectangles computed from a terminal size.
/// Pure function - no IO, no terminal access.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub file_tree: Rect,
    pub editor: Rect,
    pub git_panel: Rect,
    pub status_bar: Rect,
}

/// Fixed panel proportions.
const FILE_TREE_PERCENT: u16 = 25;
const GIT_PANEL_ROWS: u16 = 3;
const STATUS_BAR_ROWS: u16 = 1;

/// Compute the layout for a given terminal size.
/// Returns None if the terminal is too small to render.
pub fn compute(width: u16, height: u16) -> Option<Layout> {
    // Minimum usable size: at least 20 wide, 6 tall.
    if width < 20 || height < 6 {
        return None;
    }

    let bottom_rows = GIT_PANEL_ROWS + STATUS_BAR_ROWS;
    let main_height = height.saturating_sub(bottom_rows);

    let tree_width = (width * FILE_TREE_PERCENT / 100).max(1);
    let editor_width = width.saturating_sub(tree_width);

    let git_y = main_height;
    let status_y = main_height + GIT_PANEL_ROWS;

    Some(Layout {
        file_tree: Rect::new(0,          0,      tree_width,   main_height),
        editor:    Rect::new(tree_width,  0,      editor_width, main_height),
        git_panel: Rect::new(0,          git_y,   width,        GIT_PANEL_ROWS),
        status_bar: Rect::new(0,         status_y, width,       STATUS_BAR_ROWS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        compute(80, 24).unwrap()
    }

    // 1. test_layout_splits_width
    #[test]
    fn test_layout_splits_width() {
        let l = layout();
        // tree ~25% of 80 = 20, editor = 60
        assert_eq!(l.file_tree.width, 20);
        assert_eq!(l.editor.width, 60);
        // panels are side by side
        assert_eq!(l.editor.x, l.file_tree.width);
        assert_eq!(l.file_tree.x, 0);
        assert_eq!(l.file_tree.y, 0);
        assert_eq!(l.editor.y, 0);
    }

    // 2. test_layout_status_bar_height
    #[test]
    fn test_layout_status_bar_height() {
        let l = layout();
        assert_eq!(l.status_bar.height, 1);
        assert_eq!(l.status_bar.y, 23); // last row of 24
        assert_eq!(l.status_bar.x, 0);
        assert_eq!(l.status_bar.width, 80);
    }

    // 3. test_layout_git_panel_height
    #[test]
    fn test_layout_git_panel_height() {
        let l = layout();
        assert_eq!(l.git_panel.height, 3);
        assert_eq!(l.git_panel.y, 20); // 24 - 3 - 1 = 20
        assert_eq!(l.git_panel.width, 80);
    }

    #[test]
    fn test_layout_main_height() {
        let l = layout();
        // main area = 24 - 3 - 1 = 20 rows
        assert_eq!(l.file_tree.height, 20);
        assert_eq!(l.editor.height, 20);
    }

    #[test]
    fn test_layout_too_small_returns_none() {
        assert!(compute(10, 4).is_none());
        assert!(compute(0, 0).is_none());
    }

    #[test]
    fn test_layout_total_width_consistent() {
        let l = layout();
        assert_eq!(l.file_tree.width + l.editor.width, 80);
    }
}

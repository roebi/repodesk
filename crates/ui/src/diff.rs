/// A single line in a diff, classified by kind.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    /// diff --git header or @@ hunk header.
    Header,
    /// Line added in new version (starts with +).
    Added,
    /// Line removed from old version (starts with -).
    Removed,
    /// Unchanged context line (starts with space).
    Context,
}

/// A parsed diff line with its kind and content.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub content: String,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a raw git diff string into a vec of DiffLine.
/// Empty input returns empty vec.
pub fn parse_diff(raw: &str) -> Vec<DiffLine> {
    if raw.is_empty() {
        return vec![];
    }

    raw.lines()
        .map(|line| {
            let kind = if line.starts_with("diff ") || line.starts_with("@@")
                || line.starts_with("---") || line.starts_with("+++")
                || line.starts_with("index ") || line.starts_with("new file")
                || line.starts_with("deleted file") || line.starts_with("Binary")
            {
                DiffKind::Header
            } else if line.starts_with('+') {
                DiffKind::Added
            } else if line.starts_with('-') {
                DiffKind::Removed
            } else {
                DiffKind::Context
            };
            DiffLine { kind, content: line.to_string() }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

/// Render a diff panel into exactly `rows` lines of exactly `cols` chars.
/// Added lines prefixed with +, Removed with -, Headers with ~, Context with space.
pub fn render_diff_panel(lines: &[DiffLine], cols: usize, rows: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(rows);

    if lines.is_empty() {
        out.push(pad_truncate("No diff", cols));
        while out.len() < rows {
            out.push(blank(cols));
        }
        return out;
    }

    for dl in lines.iter().take(rows) {
        let prefix = match dl.kind {
            DiffKind::Added   => "+",
            DiffKind::Removed => "-",
            DiffKind::Header  => "~",
            DiffKind::Context => " ",
        };
        // Strip the git-native prefix char to avoid double prefix.
        let body = match dl.kind {
            DiffKind::Added | DiffKind::Removed | DiffKind::Context => {
                if dl.content.len() > 1 { &dl.content[1..] } else { "" }
            }
            DiffKind::Header => dl.content.as_str(),
        };
        let rendered = format!("{}{}", prefix, body);
        out.push(pad_truncate(&rendered, cols));
    }

    while out.len() < rows {
        out.push(blank(cols));
    }
    out
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

// ---------------------------------------------------------------------------
// Unit tests (pure, no IO)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. test_parse_empty_diff
    #[test]
    fn test_parse_empty_diff() {
        let result = parse_diff("");
        assert!(result.is_empty());
    }

    // 2. test_parse_header_line
    #[test]
    fn test_parse_header_line() {
        let result = parse_diff("diff --git a/src/main.rs b/src/main.rs");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DiffKind::Header);
    }

    // 3. test_parse_added_line
    #[test]
    fn test_parse_added_line() {
        let result = parse_diff("+fn foo() {}");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DiffKind::Added);
    }

    // 4. test_parse_removed_line
    #[test]
    fn test_parse_removed_line() {
        let result = parse_diff("-fn bar() {}");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DiffKind::Removed);
    }

    // 5. test_parse_context_line
    #[test]
    fn test_parse_context_line() {
        let result = parse_diff(" fn baz() {}");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DiffKind::Context);
    }

    // 6. test_parse_hunk_header
    #[test]
    fn test_parse_hunk_header() {
        let result = parse_diff("@@ -1,3 +1,4 @@ fn main()");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, DiffKind::Header);
    }

    // 7. test_parse_multiline_diff
    #[test]
    fn test_parse_multiline_diff() {
        let raw = "diff --git a/lib.rs b/lib.rs\n@@ -1,2 +1,3 @@\n fn old() {}\n+fn new() {}\n-fn gone() {}";
        let result = parse_diff(raw);
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].kind, DiffKind::Header);
        assert_eq!(result[1].kind, DiffKind::Header);
        assert_eq!(result[2].kind, DiffKind::Context);
        assert_eq!(result[3].kind, DiffKind::Added);
        assert_eq!(result[4].kind, DiffKind::Removed);
    }

    // 8. test_render_diff_panel_empty
    #[test]
    fn test_render_diff_panel_empty() {
        let lines = render_diff_panel(&[], 40, 5);
        assert_eq!(lines.len(), 5);
        assert!(lines[0].contains("No diff"), "expected 'No diff': {}", lines[0]);
    }

    // 9. test_render_diff_panel_lines
    #[test]
    fn test_render_diff_panel_lines() {
        let diff_lines = parse_diff("+fn foo() {}\n-fn bar() {}");
        let rendered = render_diff_panel(&diff_lines, 40, 5);
        assert!(rendered[0].starts_with('+'), "added line should start with +: {}", rendered[0]);
        assert!(rendered[1].starts_with('-'), "removed line should start with -: {}", rendered[1]);
    }

    // 10. test_render_diff_width
    #[test]
    fn test_render_diff_width() {
        let diff_lines = parse_diff("+fn foo() {}\n-fn bar() {}\n diff --git a/f b/f");
        let cols = 60;
        let rendered = render_diff_panel(&diff_lines, cols, 6);
        for (i, line) in rendered.iter().enumerate() {
            assert_eq!(
                line.chars().count(), cols,
                "line {} wrong width: {:?}", i, line
            );
        }
    }

    // Extra: --- and +++ lines classified as Header
    #[test]
    fn test_parse_file_markers_as_header() {
        let raw = "--- a/main.rs\n+++ b/main.rs";
        let result = parse_diff(raw);
        assert_eq!(result[0].kind, DiffKind::Header);
        assert_eq!(result[1].kind, DiffKind::Header);
    }

    // Extra: render pads short lines to cols
    #[test]
    fn test_render_diff_pads_blank_rows() {
        let rendered = render_diff_panel(&[], 30, 4);
        assert_eq!(rendered.len(), 4);
        for line in &rendered {
            assert_eq!(line.len(), 30);
        }
    }
}

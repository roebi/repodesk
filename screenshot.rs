// Standalone screenshot generator - uses snapshot engine directly.
use repodesk_commands::{Command, EditorCommand, GitCommand, LayoutCommand};
use repodesk_core::{reduce, AppState};
use repodesk_git::StatusEntry;
use repodesk_ui::snapshot::render_to_lines_with_layout;

fn main() {
    // Build a realistic AppState
    let state = AppState::default();

    // Open a file
    let state = reduce(state, Command::OpenFile("src/main.rs".to_string()));

    // Type some content into the buffer
    let lines_to_type = vec![
        "use repodesk_core::AppState;",
        "",
        "fn main() {",
        "    let state = AppState::default();",
        "    println!(\"Repodesk ready\");",
        "}",
    ];
    let mut state = state;
    for (i, line) in lines_to_type.iter().enumerate() {
        for ch in line.chars() {
            state = reduce(state, Command::Editor(EditorCommand::InsertChar(ch)));
        }
        if i < lines_to_type.len() - 1 {
            state = reduce(state, Command::Editor(EditorCommand::SplitLine));
        }
    }

    // Move cursor to line 4 col 10
    for _ in 0..3 { state = reduce(state, Command::Editor(EditorCommand::MoveUp)); }
    for _ in 0..10 { state = reduce(state, Command::Editor(EditorCommand::MoveRight)); }

    // Set git state
    state = reduce(state, Command::Git(GitCommand::RefreshBranch("main".to_string())));
    state = reduce(state, Command::Git(GitCommand::RefreshStatus(vec![
        StatusEntry { code: " M".to_string(), path: "src/main.rs".to_string() },
        StatusEntry { code: "??".to_string(), path: "src/lib.rs".to_string() },
    ])));

    // Status bar
    state = reduce(state, Command::SetStatus(
        "Ctrl-P: palette  Ctrl-S: save  Ctrl-D: diff  Ctrl-Q: quit".to_string()
    ));

    let lines = render_to_lines_with_layout(&state, 80, 24);

    // Top border
    println!("+{}+", "-".repeat(80));
    for line in &lines {
        println!("|{}|", line);
    }
    println!("+{}+", "-".repeat(80));
}

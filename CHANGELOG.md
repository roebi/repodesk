# Changelog

All notable changes to Repodesk will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned

- Phase 12: Stabilization (render latency target <16ms, panic hardening, stress tests)
- Trivy as hard CI gate (currently advisory)
- ratatui upgrade path (requires MSRV bump to 1.85)
- Buffer switching UX (Ctrl-Tab cycle)
- Line number display (config: `editor.line_numbers = true`)
- Git auto-refresh (config: `git.auto_refresh = true`)

---

## [0.1.0] - 2026-06-14

### Added

#### Core architecture (Phase 1 - AppState + Reducer)
- `AppState` struct as single source of truth for all application state
- Pure `reduce(state, command) -> AppState` function - no side effects
- `Command` enum as sole entry point for all state mutations
- Determinism guarantee: same input always produces same output
- `Buffer` and `BufferMeta` structs for multi-file buffer management
- `SetGitBranch`, `SetStatus`, `OpenFile`, `SwitchBuffer`, `CloseBuffer` commands

#### Editor buffer (Phase 2 - Buffer + Cursor)
- `Buffer` with `Vec<String>` line-based internal representation
- `Cursor` with zero-based `(row, col)` model
- Cursor movement: `move_right`, `move_left`, `move_up`, `move_down`
- Column clamping when moving to shorter lines
- `insert_char` at cursor position with cursor advance
- `delete_char` (backspace semantics) with cursor retreat
- `split_line` (Enter key) splits current line at cursor, cursor to new line
- `Buffer::from_str` for loading file content into buffer

#### Git CLI wrapper (Phase 3 - GitBackend)
- `GitBackend` trait: `current_branch`, `list_branches`, `status`, `diff`, `commit`, `checkout`
- `MockGit` in-memory implementation for unit tests (no process calls)
- `GitCli` real implementation via `std::process::Command`
- `StatusEntry` with porcelain two-character code and file path
- `GitError` enum: `NotARepo`, `CommandFailed`, `ParseError`
- Failure-safe design: all errors return `GitError`, never panic
- Integration tests against real git repositories in temp directories

#### Integration loop (Phase 4 - Full wiring)
- `EditorCommand` variants: `MoveRight`, `MoveLeft`, `MoveUp`, `MoveDown`, `InsertChar`, `DeleteChar`, `SplitLine`
- `GitCommand` variants: `RefreshBranch`, `RefreshStatus`
- `active_editor: Option<Buffer>` field in `AppState`
- `git_status: Vec<StatusEntry>` field in `AppState`
- Editor buffer persisted into buffer list on `OpenFile` and `SwitchBuffer`
- `EditorCommand` dispatched to active buffer; noop if no file open

#### Terminal UI (Phase 5 - crossterm rendering)
- ASCII terminal rendering via `crossterm 0.26` (MSRV 1.75 compatible)
- Four-panel layout: file tree (25%), editor (75%), git panel (3 rows), status bar (1 row)
- `layout::compute(width, height) -> Option<Layout>` pure layout engine
- `snapshot::render_to_lines(state, width, height) -> Vec<String>` pure renderer
- Cursor marker `|` rendered at exact cursor position in editor panel
- Crossterm raw mode, alternate screen, hide/show cursor
- Event loop: key events dispatched as `Command` through reducer
- `Ctrl-Q` quit, `Ctrl-S` save stub, arrow keys, char insert, backspace, enter

#### File system (Phase 6 - fs crate)
- `read_file(path) -> Result<Buffer, FsError>` loads file content into buffer
- `write_file(path, buffer) -> Result<(), FsError>` saves buffer to disk
- `list_dir(path) -> Result<Vec<DirEntry>, FsError>` sorted directory listing
- `entry_kind(path) -> Result<EntryKind, FsError>` file vs directory detection
- `FsError`: `NotFound`, `PermissionDenied`, `IsDirectory`, `IoError`
- File tree panel uses real `list_dir` when `repo_root` is provided
- `Ctrl-O` open file input mode, `Ctrl-S` real write via `write_file`
- Git branch and status refreshed on file open

#### Search (Phase 7 - search crate)
- `search_files(root, query) -> Result<Vec<FileMatch>, SearchError>`
- `search_content(root, query) -> Result<Vec<ContentMatch>, SearchError>`
- Exact match: case-insensitive substring (returned first)
- Fuzzy match: subsequence with consecutive run bonus scoring (fallback)
- Results sorted: Exact first (score desc), Fuzzy after (score desc)
- Hidden files and directories (leading dot) skipped
- Binary files (invalid UTF-8) skipped silently in content search
- `SearchError`: `RootNotFound`, `IoError`
- Pure std implementation, no external dependencies

#### Configuration (Phase 8 - config crate)
- `Config` struct with four sections: `[keybindings]`, `[theme]`, `[editor]`, `[git]`
- TOML loading via `toml 0.5` + `serde` (MSRV 1.75 compatible)
- Config file location: `.repodesk/config.toml` in repo root
- `Config::load(repo_root) -> Result<Config, ConfigError>`
- `Config::load_or_default(repo_root) -> Config` (never fails)
- Partial config: missing fields fall back to defaults via `serde(default)`
- `keybinding_for(action) -> Option<&str>` lookup by action name
- `ConfigError`: `NotFound`, `ParseError`, `IoError`
- Config loaded at startup and drives keybinding dispatch in event loop

#### Diff view (Phase 9 - diff panel)
- `GitBackend::diff_cached()` returns staged diff
- `GitBackend::diff_file(path)` returns single-file diff
- `MockGit::cached_diff_output` field for testing
- `GitCommand::RefreshDiff(String)` command
- `AppState::diff_output: Option<String>` field
- `diff::parse_diff(raw) -> Vec<DiffLine>` pure parser
- `DiffKind`: `Header`, `Added`, `Removed`, `Context`
- `diff::render_diff_panel(lines, cols, rows) -> Vec<String>` renderer
- `+` prefix for added lines, `-` for removed, `~` for headers

#### Command palette and layout management (Phase 10)
- `PaletteCommand`: `Open`, `Close`, `SetQuery`, `SelectResult`, `MoveSelection`
- `LayoutCommand`: `ToggleDiff`, `ToggleEditorFull`, `ResetLayout`
- `LayoutMode` enum: `Normal`, `DiffOpen`, `EditorFull`
- `AppState` fields: `palette_open`, `palette_query`, `palette_results`, `palette_selected`, `layout_mode`
- `builtin_palette_entries()` with six built-in actions
- `filter_palette(query)` case-insensitive substring filter
- `palette::render_palette_overlay` centred overlay renderer
- `snapshot::render_to_lines_with_layout` layout-mode-aware renderer
- `EditorFull` mode: tree and git panel hidden, editor takes full width
- `DiffOpen` mode: git panel replaced by diff panel
- `Ctrl-P` opens palette, `Ctrl-D` toggles diff, `Ctrl-E` toggles editor full

#### Distribution and deployment (Phase 11)
- Multi-stage `Dockerfile`: Rust 1.75 builder + Alpine 3.19 runtime
- Non-root container user (`repodesk`)
- `entrypoint.sh`: workspace validation, git identity init, `safe.directory` config
- ShellCheck-verified `entrypoint.sh` (0 warnings)
- `.github/workflows/ci.yml`: `cargo test`, release build, ShellCheck on Rust 1.75 and stable
- `.github/workflows/release.yml`: tag-triggered musl static binary artifact
- `.github/workflows/docker.yml`: ghcr.io push, Trivy advisory scan on CRITICAL/HIGH

### Changed

- Nothing (initial release)

### Deprecated

- Nothing

### Removed

- Nothing

### Fixed

- `OpenFile` command now flushes active editor state into buffer list before switching,
  fixing a regression where buffer content was lost when opening a second file

### Security

- All GitHub Actions use SHA-pinned action versions
- Trivy vulnerability scan on every container image build (advisory in v0.1.0)
- Non-root user in container image
- `entrypoint.sh` uses `set -e` for fail-fast behavior

---

[Unreleased]: https://github.com/roebi/repodesk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/roebi/repodesk/releases/tag/v0.1.0

# Repodesk — Implementation Plan (Full Ledger)

This document defines the full execution sequence of the system.

Each phase is explicitly defined and must be executed in order using TDD.

---

## Phase 2 — Requirements Definition

- Product scope defined
- Execution environment defined
- Functional requirements defined
- Constraints defined
- Architecture principles defined

Output: requirements.md (approved)

---

## Phase 3 — System Architecture

### Architecture Decisions

- Single binary Rust application
- State-driven architecture
- Reducer-based command system
- UI is pure rendering layer

### Module Layout

- core/
- ui/
- editor/
- git/
- fs/
- commands/
- search/
- config/

### Core Loop

Input -> Command -> Reducer -> State -> Render

---

## Phase 4 — TDD Design Specification

### Initial Test Set

- App initialization with repo path
- File tree loading
- Buffer creation
- Insert/delete operations
- File save
- Git branch detection
- Branch listing
- Checkout operation

---

## Phase 5 — Implementation Order

1. Cargo project initialization
2. AppState + initial tests
3. File system module
4. Editor buffer implementation
5. Git CLI wrapper
6. Command system
7. UI (crossterm - MSRV 1.75 compatible; ratatui requires 1.85+)
8. Input handling
9. Full integration loop

---

## Phase 6 — First TDD Cycle

- Create AppState
- Write failing test
- Implement minimal functionality
- Make test pass
- Iterate incrementally

---

## Phase 7 — UI Integration

- crossterm rendering layer (ratatui blocked by MSRV 1.75 constraint)
- Render file tree
- Render editor view
- Render Git status
- Implement input -> command mapping
- Establish event loop

---

## Phase 8 — Interactive Editor System

- Cursor model implementation
- Insert/delete operations
- Multi-buffer system
- File open/save lifecycle
- Editor state synchronization

---

## Phase 9 — Git Integration Layer

- Branch listing support
- Checkout functionality
- Status display integration
- Diff view (basic text diff) - parse_diff + render_diff_panel
- diff_cached() and diff_file(path) on GitBackend trait
- Commit workflow integration

---

## Phase 10 — Productivity Layer

- Command palette system (PaletteCommand, builtin_palette_entries)
- Fuzzy search in repository (search_files, search_content)
- Layout management system (LayoutMode: Normal, DiffOpen, EditorFull)
- Buffer switching UX improvements
- Navigation enhancements

---

## Phase 11 — Distribution and Deployment

### Outputs

- Rust binary (`cargo install repodesk`)
- Podman/Docker container image (musl static binary on Alpine 3.19)
- SSH entrypoint execution model

### Runtime

- entrypoint.sh initializes IDE
- repository mounted into /workspace
- deterministic startup behavior

### CI/CD

- cargo test (Rust 1.75 + stable matrix)
- cargo build --release (musl target)
- ShellCheck on entrypoint.sh and podman scripts
- docker/podman build
- artifact release pipeline (tag-triggered)
- Trivy vulnerability scan (advisory in Phase 11)

### Known Issues Resolved

- rust:1.75-slim builder produces glibc binary incompatible with Alpine musl
  -> Fix: rustup target add x86_64-unknown-linux-musl + --target flag
- `file` utility not available on Alpine
  -> Fix: removed from Dockerfile verification step
- entrypoint.sh used bare `repodesk` command
  -> Fix: absolute path /usr/local/bin/repodesk

---

## Phase 12 — Stabilization (next session)

### Performance

- UI render optimization (<16ms target)
- File tree caching
- Buffer efficiency improvements

### Stability

- Git failure isolation
- IO error recovery
- No panic in runtime paths

### Stress Testing

- Large repositories
- Long sessions (hours)
- Rapid input scenarios
- Frequent Git operations

### Known Candidates from Phase 11 Runtime

- Hidden files (.git, .aider.*) visible in file tree
  -> Candidate: dot-file filter in render_file_tree in snapshot.rs
- Trivy scan upgrade from advisory to hard CI gate
- ratatui upgrade path (requires MSRV bump to 1.85)
- Buffer switching UX (Ctrl-Tab cycle)
- Line number display (config: editor.line_numbers = true)
- Git auto-refresh (config: git.auto_refresh = true)

---

## Implementation Status

| Phase | Description              | Status   | Tests |
|-------|--------------------------|----------|-------|
| 2     | Requirements             | Done     | -     |
| 3     | Architecture             | Done     | -     |
| 4     | TDD Design               | Done     | -     |
| 5     | Implementation Order     | Done     | -     |
| 6     | First TDD Cycle          | Done     | 14    |
| 7     | UI Integration           | Done     | 17    |
| 8     | Interactive Editor       | Done     | 20    |
| 9     | Git Integration + Diff   | Done     | 19    |
| 10    | Productivity Layer       | Done     | 25    |
| 11    | Distribution             | Done     | 0*    |
| 12    | Stabilization            | Pending  | -     |

*Phase 11 has no new Rust tests - all gates run existing 189-test suite.

Total tests as of Phase 11: 189 across 8 crates, all green.

---

## Final System Goal

Repodesk becomes:

> a stable, deterministic, Rust-based terminal IDE optimized for Git workflows
> in SSH/container environments

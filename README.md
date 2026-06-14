# Repodesk

A deterministic, Rust-based terminal IDE for Git workflows in SSH and container environments.

```
+--------------------------------------------------------------------------------+
|FILES               use repodesk_core::AppState;                                |
|> src/main.rs                                                                   |
|                    fn main() {                                                 |
|                        let state = App|State::default();                       |
|                        println!("Repodesk ready");                             |
|                    }                                                           |
|                                                                                |
|GIT: branch=main                                                                |
|   M src/main.rs                                                                |
|  ?? Cargo.lock                                                                 |
|Ctrl-P:palette  Ctrl-S:save  Ctrl-D:diff  Ctrl-O:open  Ctrl-Q:quit              |
+--------------------------------------------------------------------------------+
```

---

## Features

- ASCII terminal rendering - works over SSH and inside containers
- State-driven architecture with pure reducer (no UI-driven mutations)
- File tree panel, editor panel, git panel, diff panel
- Command palette with fuzzy filtering
- Three layout modes: Normal, Diff Open, Editor Full
- Git integration: branch, status, diff, commit, checkout
- Repository-wide file and content search (exact + fuzzy)
- Configurable keybindings via `.repodesk/config.toml`
- Single static binary, no runtime dependencies except `git`

---

## Architecture

Repodesk is a Cargo workspace with strict separation of concerns:

```
Input -> Command -> Reducer -> AppState -> Render
```

```
crates/
  commands/   Command enum, EditorCommand, GitCommand, PaletteCommand, LayoutCommand
  core/       AppState, pure reducer, LayoutMode, PaletteResult
  editor/     Buffer, Cursor (Vec<String> line model)
  git/        GitBackend trait, MockGit (tests), GitCli (real process wrapper)
  fs/         read_file, write_file, list_dir, entry_kind
  search/     search_files, search_content (exact + fuzzy subsequence scoring)
  config/     Config, TOML loader (.repodesk/config.toml)
  ui/         layout, snapshot, diff, palette, app (crossterm event loop)
```

- MSRV: Rust 1.75 (matches Ubuntu 24.04, container-safe)
- No unsafe code
- 189+ tests across all crates

---

## Keybindings

| Key        | Action                        |
|------------|-------------------------------|
| Ctrl-P     | Open command palette          |
| Ctrl-O     | Open file (:open input mode)  |
| Ctrl-S     | Save active buffer            |
| Ctrl-G     | Refresh git branch and status |
| Ctrl-D     | Toggle diff panel             |
| Ctrl-E     | Toggle editor full mode       |
| Ctrl-Q     | Quit                          |
| Arrows     | Move cursor                   |
| Backspace  | Delete character              |
| Enter      | Split line                    |
| Esc        | Close palette / cancel input  |

### Palette navigation

| Key        | Action                        |
|------------|-------------------------------|
| Up / Down  | Move selection                |
| Enter      | Execute selected entry        |
| Esc        | Close palette                 |
| Any char   | Filter results                |

---

## Build

### From source

Requires Rust 1.75 or newer and `git` on PATH.

```sh
cargo build --release
./target/release/repodesk [path/to/repo]
```

Defaults to current directory if no path is given.

### Container (Podman)

```sh
# Build image
./podman_build.sh

# Run with current directory as workspace
./podman_run.sh

# Run with explicit repository path
./podman_run.sh /path/to/repo
```

---

## Container Usage

The container expects a Git repository mounted at `/workspace`.

```sh
podman run -it --rm \
  -v /path/to/repo:/workspace:z \
  repodesk:latest
```

The `:z` flag is required on SELinux systems (Fedora, RHEL).

---

## Configuration

Create `.repodesk/config.toml` in your repository root:

```toml
[keybindings]
quit        = "ctrl-q"
save        = "ctrl-s"
open        = "ctrl-o"
refresh_git = "ctrl-g"

[theme]
show_git_panel    = true
show_line_numbers = false

[editor]
tab_width    = 4
line_numbers = false

[git]
auto_refresh      = false
auto_refresh_secs = 30
```

All fields are optional. Missing fields fall back to the defaults shown above.

---

## Layout Modes

| Mode          | Key    | Description                              |
|---------------|--------|------------------------------------------|
| Normal        | -      | File tree + editor + git panel           |
| Diff Open     | Ctrl-D | Git panel replaced by diff panel         |
| Editor Full   | Ctrl-E | Tree and git panel hidden, editor full   |

---

## Screenshots

See [repodesk-screenshots.md](repodesk-screenshots.md) for all four layout modes
rendered at 80x24.

---

## Testing

```sh
# Run all tests
cargo test --workspace

# Run a specific crate
cargo test -p repodesk-core
cargo test -p repodesk-git
```

Integration tests require `git` on PATH and write to `$TMPDIR`.

---

## CI

GitHub Actions workflows in `.github/workflows/`:

| Workflow       | Trigger              | Gates                              |
|----------------|----------------------|------------------------------------|
| `ci.yml`       | push / PR to main    | cargo test, release build, ShellCheck |
| `release.yml`  | tag `v*.*.*`         | musl static binary artifact        |
| `docker.yml`   | push / tag           | image build, push to ghcr.io, Trivy scan |

---

## License

MIT

# Repodesk — Requirements Specification (v1.0)

## 0. System Definition

Repodesk is a Rust-based terminal IDE designed for Git repository workflows.

It operates exclusively in ASCII terminal environments, primarily over SSH connections and inside Podman/Docker containers.

The system is single-user and state-driven.

---

## 1. Product Scope

Repodesk provides a full development environment inside a terminal:

- File navigation
- Code editing
- Git workflows
- Search capabilities
- Command-driven interaction model
- IDE-like layout in ASCII terminal

---

## 2. Execution Environment

- SSH-based access supported
- Container-first deployment (Podman/Docker)
- Offline capable after installation
- Deterministic runtime behavior across environments
- Linux-first design assumption

---

## 3. Interaction Model

- Continuous, reactive UI (VSCode-like behavior)
- Keyboard-driven interface
- No strict modal dependency required
- Command palette as primary discovery mechanism

---

## 4. Functional Requirements

### 4.1 File System

- Repository tree visualization
- Directory traversal
- File open/read/write operations
- Buffer management for open files

---

### 4.2 Editor

- Cursor movement (row/column model)
- Insert and delete operations
- Multi-buffer support (tabs)
- Save and reload functionality
- Plain text editing for programming languages

---

### 4.3 Git Integration

- Current branch detection
- Branch listing
- Branch checkout
- Git status visualization
- Diff display (text-based)
- Commit execution from within IDE

All Git operations are initially implemented via CLI wrapper (`git` command).

---

### 4.4 UI System

- ASCII terminal rendering
- Split-pane layout:
  - file tree panel
  - editor panel
  - status panel
  - optional git panel
- Dynamic layout toggling

---

### 4.5 Command System

- Central command abstraction layer
- Command palette (interactive)
- Keybinding -> Command mapping system
- All actions routed through command reducer

---

### 4.6 Search System

- Repository-wide file search
- Fuzzy matching support (future-facing requirement)

---

## 5. Architecture Requirements

### 5.1 State Model

- Single central `AppState`
- All mutations occur via reducer pattern
- UI is a pure function of state

---

### 5.2 Core Design Principles

- Deterministic state transitions
- No UI-driven logic mutation
- No direct side effects in UI layer
- Strict separation of concerns

---

### 5.3 Module Structure (Rust)

- core/
- ui/
- editor/
- git/
- fs/
- commands/
- search/
- config/

---

## 6. Editing Engine Requirements

- Line-based internal representation preferred
- Cursor-based navigation model
- Efficient insert/delete operations
- Buffer isolation per open file

---

## 7. Git System Requirements

- External process wrapper (initially)
- No dependency on git2 in early phases
- Failure-safe design (no UI crashes on Git failure)
- Non-blocking Git operations

---

## 8. Container Deployment Requirements

- Single container runtime
- entrypoint.sh launches IDE
- Repository mounted into container workspace
- Deterministic startup behavior

---

## 9. Stability Requirements

- No runtime crashes in normal operation
- UI must remain responsive under load
- Long-running SSH sessions supported
- Git failures must not terminate application
- Safe recovery from IO errors required

---

## 10. Testing Requirements (TDD Mandatory)

- Core system fully test-driven
- UI minimally tested initially
- Git integration tested via integration tests where feasible
- Strict cycle:
  test -> implement -> verify -> refactor

---

## 11. Non-Functional Requirements

- Low latency terminal interaction
- Predictable memory usage
- Fast file tree rendering for large repositories
- Stable long session behavior

---

## 12. Final System Goal

Repodesk becomes:

> a deterministic Rust-based terminal IDE for Git workflows in SSH/container environments

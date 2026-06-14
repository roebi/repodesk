# Repodesk - ASCII Screenshots

Generated live from the snapshot engine (`crates/ui/src/snapshot.rs`).
Terminal size: 80x24. Cursor position marked with `|`.

---

## 1. Normal View

File tree (left 25%) + editor (right 75%) + git panel + status bar.

```
+--------------------------------------------------------------------------------+
|FILES               use repodesk_core::AppState;                                |
|> src/main.rs                                                                   |
|                    fn main() {                                                 |
|                        let state = App|State::default();                       |
|                        println!("Repodesk ready");                             |
|                    }                                                           |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|GIT: branch=main                                                                |
|   M src/main.rs                                                                |
|  ?? Cargo.lock                                                                 |
|Ctrl-P:palette  Ctrl-S:save  Ctrl-D:diff  Ctrl-O:open  Ctrl-Q:quit              |
+--------------------------------------------------------------------------------+
```

---

## 2. Palette Overlay (Ctrl-P, query: 'git')

Command palette centred overlay. Query filters results in real time.
`>` marks the selected entry. Background content remains visible.

```
+--------------------------------------------------------------------------------+
|FILES               use repodesk_core::AppState;                                |
|> src/main.rs                                                                   |
|              +--------------------------------------------------+              |
|              |> git                                             |              |
|              |--------------------------------------------------|              |
|              |> Refresh Git                                     |              |
|              +--------------------------------------------------+              |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|GIT: branch=main                                                                |
|   M src/main.rs                                                                |
|  ?? Cargo.lock                                                                 |
|Palette open - type to filter                                                   |
+--------------------------------------------------------------------------------+
```

---

## 3. Diff View (Ctrl-D)

Git panel replaced by diff panel. `~` = header line, `+` = added, `-` = removed.
Toggle back with Ctrl-D.

```
+--------------------------------------------------------------------------------+
|FILES               use repodesk_core::AppState;                                |
|> src/main.rs                                                                   |
|                    fn main() {                                                 |
|                        let state = App|State::default();                       |
|                        println!("Repodesk ready");                             |
|                    }                                                           |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|~diff --git a/src/main.rs b/src/main.rs                                         |
|~index abc..def 100644                                                          |
|~--- a/src/main.rs                                                              |
|Diff view active (Ctrl-D to toggle back)                                        |
+--------------------------------------------------------------------------------+
```

---

## 4. Editor Full Mode (Ctrl-E)

File tree and git panel hidden. Editor takes full terminal width.
Toggle back with Ctrl-E.

```
+--------------------------------------------------------------------------------+
|use repodesk_core::AppState;                                                    |
|                                                                                |
|fn main() {                                                                     |
|    let state = App|State::default();                                           |
|    println!("Repodesk ready");                                                 |
|}                                                                               |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|                                                                                |
|Editor full mode (Ctrl-E to toggle back)                                        |
+--------------------------------------------------------------------------------+
```

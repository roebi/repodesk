/// Integration tests for repodesk-config.
/// All tests use real temp directories.
use repodesk_config::{Config, ConfigError};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir().join(format!("repodesk-cfg-{}-{}", label, ns));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_config(repo_root: &PathBuf, content: &str) {
    let cfg_dir = repo_root.join(".repodesk");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(cfg_dir.join("config.toml"), content).unwrap();
}

// ---------------------------------------------------------------------------
// 11. test_load_missing_file
// ---------------------------------------------------------------------------

#[test]
fn test_load_missing_file() {
    let dir = temp_dir("missing");
    // No .repodesk/config.toml created.
    let result = Config::load(&dir);
    assert_eq!(result, Err(ConfigError::NotFound));
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 12. test_load_or_default_missing
// ---------------------------------------------------------------------------

#[test]
fn test_load_or_default_missing() {
    let dir = temp_dir("or-default");
    // No config file -> silently returns defaults.
    let cfg = Config::load_or_default(&dir);
    assert_eq!(cfg, Config::default());
    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 13. test_load_full_config
// ---------------------------------------------------------------------------

#[test]
fn test_load_full_config() {
    let dir = temp_dir("full");
    write_config(&dir, r#"
[keybindings]
quit = "ctrl-z"
save = "ctrl-w"
open = "ctrl-p"
refresh_git = "ctrl-r"

[theme]
show_line_numbers = true
show_git_panel = false

[editor]
tab_width = 2
line_numbers = true

[git]
auto_refresh = true
auto_refresh_secs = 15
"#);

    let cfg = Config::load(&dir).unwrap();
    assert_eq!(cfg.keybindings.quit, "ctrl-z");
    assert_eq!(cfg.keybindings.save, "ctrl-w");
    assert_eq!(cfg.keybindings.open, "ctrl-p");
    assert_eq!(cfg.keybindings.refresh_git, "ctrl-r");
    assert!(cfg.theme.show_line_numbers);
    assert!(!cfg.theme.show_git_panel);
    assert_eq!(cfg.editor.tab_width, 2);
    assert!(cfg.editor.line_numbers);
    assert!(cfg.git.auto_refresh);
    assert_eq!(cfg.git.auto_refresh_secs, 15);

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 14. test_load_partial_config
// ---------------------------------------------------------------------------

#[test]
fn test_load_partial_config() {
    let dir = temp_dir("partial");
    // Only override one field - all others fall back to defaults.
    write_config(&dir, r#"
[editor]
tab_width = 2
"#);

    let cfg = Config::load(&dir).unwrap();
    assert_eq!(cfg.editor.tab_width, 2);
    // Unspecified fields use defaults.
    assert!(!cfg.editor.line_numbers);
    assert_eq!(cfg.keybindings.quit, "ctrl-q");
    assert!(cfg.theme.show_git_panel);
    assert!(!cfg.git.auto_refresh);

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 15. test_load_invalid_toml_file
// ---------------------------------------------------------------------------

#[test]
fn test_load_invalid_toml_file() {
    let dir = temp_dir("invalid");
    write_config(&dir, "[[[[this is not valid toml");

    let result = Config::load(&dir);
    assert!(
        matches!(result, Err(ConfigError::ParseError(_))),
        "expected ParseError, got: {:?}", result
    );

    fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------
// 16. test_load_config_path
// ---------------------------------------------------------------------------

#[test]
fn test_load_config_path() {
    let dir = temp_dir("path");
    // Verify the expected path is .repodesk/config.toml.
    let expected = dir.join(".repodesk").join("config.toml");
    assert_eq!(Config::config_path(&dir), expected);

    // Write to that exact path and confirm load works.
    write_config(&dir, "[editor]\ntab_width = 8\n");
    let cfg = Config::load(&dir).unwrap();
    assert_eq!(cfg.editor.tab_width, 8);

    fs::remove_dir_all(&dir).ok();
}

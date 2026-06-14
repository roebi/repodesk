use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Config file does not exist.
    NotFound,
    /// File exists but is not valid TOML or does not match the schema.
    ParseError(String),
    /// OS-level IO error.
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "Config file not found"),
            ConfigError::ParseError(msg) => write!(f, "Config parse error: {}", msg),
            ConfigError::IoError(msg) => write!(f, "Config IO error: {}", msg),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            ConfigError::NotFound
        } else {
            ConfigError::IoError(e.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-config structs
// ---------------------------------------------------------------------------

/// Keybinding overrides. Values are strings like "ctrl-q", "ctrl-s".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    pub quit: String,
    pub save: String,
    pub open: String,
    pub refresh_git: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            quit:        "ctrl-q".to_string(),
            save:        "ctrl-s".to_string(),
            open:        "ctrl-o".to_string(),
            refresh_git: "ctrl-g".to_string(),
        }
    }
}

/// Theme / display settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub show_line_numbers: bool,
    pub show_git_panel: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            show_git_panel: true,
        }
    }
}

/// Editor behaviour settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub tab_width: u8,
    pub line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_numbers: false,
        }
    }
}

/// Git integration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub auto_refresh: bool,
    pub auto_refresh_secs: u32,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_refresh: false,
            auto_refresh_secs: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level Config
// ---------------------------------------------------------------------------

/// Full Repodesk configuration.
/// Loaded from `.repodesk/config.toml` in the repo root.
/// Missing sections or fields fall back to their defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub keybindings: KeybindingsConfig,
    pub theme: ThemeConfig,
    pub editor: EditorConfig,
    pub git: GitConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybindings: KeybindingsConfig::default(),
            theme:       ThemeConfig::default(),
            editor:      EditorConfig::default(),
            git:         GitConfig::default(),
        }
    }
}

impl Config {
    /// Standard config file path relative to repo root.
    pub fn config_path(repo_root: &Path) -> std::path::PathBuf {
        repo_root.join(".repodesk").join("config.toml")
    }

    /// Load config from `.repodesk/config.toml` inside `repo_root`.
    /// Returns `ConfigError::NotFound` if the file does not exist.
    pub fn load(repo_root: &Path) -> Result<Self, ConfigError> {
        let path = Self::config_path(repo_root);

        if !path.exists() {
            return Err(ConfigError::NotFound);
        }

        let content = std::fs::read_to_string(&path)?;
        Self::from_str(&content)
    }

    /// Parse a TOML string into a Config.
    /// Missing fields are filled with defaults via serde(default).
    pub fn from_str(toml_str: &str) -> Result<Self, ConfigError> {
        toml::from_str(toml_str).map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load config or fall back to defaults without ever failing.
    /// Suitable for startup where a missing config is normal.
    pub fn load_or_default(repo_root: &Path) -> Self {
        match Self::load(repo_root) {
            Ok(cfg) => cfg,
            Err(_) => Self::default(),
        }
    }

    /// Look up the configured key string for a named action.
    /// Returns None for unknown action names.
    pub fn keybinding_for(&self, action: &str) -> Option<&str> {
        match action {
            "quit"        => Some(&self.keybindings.quit),
            "save"        => Some(&self.keybindings.save),
            "open"        => Some(&self.keybindings.open),
            "refresh_git" => Some(&self.keybindings.refresh_git),
            _             => None,
        }
    }

    /// Return all keybindings as a map of action -> key string.
    pub fn all_keybindings(&self) -> HashMap<&str, &str> {
        let mut map = HashMap::new();
        map.insert("quit",        self.keybindings.quit.as_str());
        map.insert("save",        self.keybindings.save.as_str());
        map.insert("open",        self.keybindings.open.as_str());
        map.insert("refresh_git", self.keybindings.refresh_git.as_str());
        map
    }
}

// ---------------------------------------------------------------------------
// Unit tests - pure struct / TOML string parsing, no IO
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. test_default_config_quit_key
    #[test]
    fn test_default_config_quit_key() {
        let cfg = Config::default();
        assert_eq!(cfg.keybindings.quit, "ctrl-q");
    }

    // 2. test_default_config_tab_width
    #[test]
    fn test_default_config_tab_width() {
        let cfg = Config::default();
        assert_eq!(cfg.editor.tab_width, 4);
    }

    // 3. test_default_config_line_numbers
    #[test]
    fn test_default_config_line_numbers() {
        let cfg = Config::default();
        assert!(!cfg.editor.line_numbers);
    }

    // 4. test_default_config_show_git
    #[test]
    fn test_default_config_show_git() {
        let cfg = Config::default();
        assert!(cfg.theme.show_git_panel);
    }

    // 5. test_default_config_auto_refresh
    #[test]
    fn test_default_config_auto_refresh() {
        let cfg = Config::default();
        assert!(!cfg.git.auto_refresh);
        assert_eq!(cfg.git.auto_refresh_secs, 30);
    }

    // 6. test_keybinding_for_known_action
    #[test]
    fn test_keybinding_for_known_action() {
        let cfg = Config::default();
        assert_eq!(cfg.keybinding_for("quit"),        Some("ctrl-q"));
        assert_eq!(cfg.keybinding_for("save"),        Some("ctrl-s"));
        assert_eq!(cfg.keybinding_for("open"),        Some("ctrl-o"));
        assert_eq!(cfg.keybinding_for("refresh_git"), Some("ctrl-g"));
    }

    // 7. test_keybinding_for_unknown
    #[test]
    fn test_keybinding_for_unknown() {
        let cfg = Config::default();
        assert_eq!(cfg.keybinding_for("unknown"), None);
        assert_eq!(cfg.keybinding_for(""), None);
    }

    // 8. test_parse_toml_overrides_quit
    #[test]
    fn test_parse_toml_overrides_quit() {
        let toml = r#"
[keybindings]
quit = "ctrl-x"
"#;
        let cfg = Config::from_str(toml).unwrap();
        assert_eq!(cfg.keybindings.quit, "ctrl-x");
        // other keys still default
        assert_eq!(cfg.keybindings.save, "ctrl-s");
    }

    // 9. test_parse_toml_overrides_tab
    #[test]
    fn test_parse_toml_overrides_tab() {
        let toml = r#"
[editor]
tab_width = 2
line_numbers = true
"#;
        let cfg = Config::from_str(toml).unwrap();
        assert_eq!(cfg.editor.tab_width, 2);
        assert!(cfg.editor.line_numbers);
        // git section still default
        assert!(!cfg.git.auto_refresh);
    }

    // 10. test_parse_invalid_toml
    #[test]
    fn test_parse_invalid_toml() {
        let bad = "[[[[not valid toml";
        let result = Config::from_str(bad);
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    // Extra: parse full config
    #[test]
    fn test_parse_full_config() {
        let toml = r#"
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
auto_refresh_secs = 10
"#;
        let cfg = Config::from_str(toml).unwrap();
        assert_eq!(cfg.keybindings.quit, "ctrl-z");
        assert_eq!(cfg.keybindings.save, "ctrl-w");
        assert!(cfg.theme.show_line_numbers);
        assert!(!cfg.theme.show_git_panel);
        assert_eq!(cfg.editor.tab_width, 2);
        assert!(cfg.editor.line_numbers);
        assert!(cfg.git.auto_refresh);
        assert_eq!(cfg.git.auto_refresh_secs, 10);
    }

    // Extra: all_keybindings returns complete map
    #[test]
    fn test_all_keybindings_map() {
        let cfg = Config::default();
        let map = cfg.all_keybindings();
        assert_eq!(map.len(), 4);
        assert_eq!(map["quit"], "ctrl-q");
        assert_eq!(map["save"], "ctrl-s");
    }
}

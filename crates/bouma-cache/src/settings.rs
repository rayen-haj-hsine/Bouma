//! Application settings (persisted to `settings.json`).

use bouma_core::sort::{SortField, SortOrder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Application settings, serialized to JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Show hidden files by default.
    #[serde(default)]
    pub show_hidden: bool,

    /// Default sort field.
    #[serde(default = "default_sort_field")]
    pub sort_field: SortField,

    /// Default sort order.
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,

    /// Sidebar width in pixels.
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,

    /// Starting directory when the app launches.
    #[serde(default = "default_start_directory")]
    pub start_directory: PathBuf,

    /// Pinned sidebar favorites.
    #[serde(default)]
    pub favorites: Vec<PathBuf>,
}

fn default_sort_field() -> SortField {
    SortField::Name
}
fn default_sort_order() -> SortOrder {
    SortOrder::Ascending
}
fn default_sidebar_width() -> u16 {
    220
}
fn default_start_directory() -> PathBuf {
    dirs_home().unwrap_or_else(|| PathBuf::from("C:\\"))
}

fn dirs_home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().to_path_buf())
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            show_hidden: false,
            sort_field: default_sort_field(),
            sort_order: default_sort_order(),
            sidebar_width: default_sidebar_width(),
            start_directory: default_start_directory(),
            favorites: Vec::new(),
        }
    }
}

impl Settings {
    /// Returns the path to the settings file: `%APPDATA%/Bouma/settings.json`.
    pub fn config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "Bouma")
            .map(|dirs| dirs.config_dir().join("settings.json"))
    }

    /// Loads settings from disk, or returns defaults if the file doesn't exist.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            warn!("Could not determine config directory, using defaults");
            return Settings::default();
        };

        Self::load_from(&path)
    }

    /// Loads settings from a specific path (useful for testing).
    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => {
                    info!("Loaded settings from {}", path.display());
                    settings
                }
                Err(err) => {
                    warn!(
                        "Failed to parse settings from {}: {}, using defaults",
                        path.display(),
                        err
                    );
                    Settings::default()
                }
            },
            Err(_) => {
                info!(
                    "No settings file at {}, using defaults",
                    path.display()
                );
                Settings::default()
            }
        }
    }

    /// Saves settings to disk.
    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_path() else {
            return Err("Could not determine config directory".to_string());
        };

        self.save_to(&path)
    }

    /// Saves settings to a specific path (useful for testing).
    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;

        fs::write(path, json).map_err(|e| format!("Failed to write settings: {e}"))?;

        info!("Saved settings to {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(!settings.show_hidden);
        assert_eq!(settings.sort_field, SortField::Name);
        assert_eq!(settings.sort_order, SortOrder::Ascending);
        assert_eq!(settings.sidebar_width, 220);
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("bouma_test_settings");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("settings.json");

        let mut settings = Settings::default();
        settings.show_hidden = true;
        settings.sort_field = SortField::Size;
        settings.favorites = vec![PathBuf::from("C:\\Projects")];

        settings.save_to(&path).unwrap();

        let loaded = Settings::load_from(&path);
        assert!(loaded.show_hidden);
        assert_eq!(loaded.sort_field, SortField::Size);
        assert_eq!(loaded.favorites.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_missing_file() {
        let settings = Settings::load_from(Path::new("C:\\nonexistent_bouma_test\\settings.json"));
        // Should return defaults, not error
        assert!(!settings.show_hidden);
    }

    #[test]
    fn test_load_corrupt_file() {
        let dir = std::env::temp_dir().join("bouma_test_corrupt");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, "this is not json!!!").unwrap();

        let settings = Settings::load_from(&path);
        // Should return defaults, not error
        assert!(!settings.show_hidden);

        let _ = fs::remove_dir_all(&dir);
    }
}

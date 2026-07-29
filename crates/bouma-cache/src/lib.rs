//! # bouma-cache
//!
//! Local persistence for the Bouma file manager.
//!
//! Stores settings, folder history, and favorites in `%APPDATA%/Bouma/`.
//! No network, no cloud — everything stays on disk.

pub mod history;
pub mod settings;

pub use history::HistoryStore;
pub use settings::Settings;

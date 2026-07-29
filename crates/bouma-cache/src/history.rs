//! Folder navigation history.
//!
//! Tracks the back/forward navigation stack, plus recently visited folders.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Maximum number of entries in the recent folders list.
const MAX_RECENT: usize = 50;

/// Manages folder navigation history (back/forward) and recent folders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStore {
    /// Back stack (most recent at the end).
    #[serde(skip)]
    back: Vec<PathBuf>,

    /// Forward stack (most recent at the end).
    #[serde(skip)]
    forward: Vec<PathBuf>,

    /// Recently visited folders (most recent at the front).
    pub recent: Vec<PathBuf>,
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryStore {
    /// Creates a new, empty history store.
    pub fn new() -> Self {
        HistoryStore {
            back: Vec::new(),
            forward: Vec::new(),
            recent: Vec::new(),
        }
    }

    /// Records a navigation to a new directory.
    ///
    /// Pushes the current directory onto the back stack and clears the forward stack
    /// (just like a web browser).
    pub fn navigate(&mut self, current: &Path, new: PathBuf) {
        self.back.push(current.to_path_buf());
        self.forward.clear();
        self.add_recent(&new);
    }

    /// Goes back to the previous directory.
    ///
    /// Returns the path to go back to, or `None` if there's no history.
    pub fn go_back(&mut self, current: &Path) -> Option<PathBuf> {
        let prev = self.back.pop()?;
        self.forward.push(current.to_path_buf());
        Some(prev)
    }

    /// Goes forward to the next directory.
    ///
    /// Returns the path to go forward to, or `None` if there's no forward history.
    pub fn go_forward(&mut self, current: &Path) -> Option<PathBuf> {
        let next = self.forward.pop()?;
        self.back.push(current.to_path_buf());
        Some(next)
    }

    /// Whether there are entries in the back stack.
    pub fn can_go_back(&self) -> bool {
        !self.back.is_empty()
    }

    /// Whether there are entries in the forward stack.
    pub fn can_go_forward(&self) -> bool {
        !self.forward.is_empty()
    }

    /// Adds a path to the recently visited list.
    fn add_recent(&mut self, path: &Path) {
        // Remove if already present (to move it to front)
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());

        // Trim to max size
        if self.recent.len() > MAX_RECENT {
            self.recent.truncate(MAX_RECENT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_and_go_back() {
        let mut history = HistoryStore::new();
        let home = PathBuf::from("C:\\Users\\rayen");
        let docs = PathBuf::from("C:\\Users\\rayen\\Documents");
        let pics = PathBuf::from("C:\\Users\\rayen\\Pictures");

        // Navigate: home → docs → pics
        history.navigate(&home, docs.clone());
        history.navigate(&docs, pics.clone());

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());

        // Go back: pics → docs
        let back = history.go_back(&pics).unwrap();
        assert_eq!(back, docs);
        assert!(history.can_go_forward());

        // Go back: docs → home
        let back = history.go_back(&docs).unwrap();
        assert_eq!(back, home);

        // Can't go back further
        assert!(!history.can_go_back());
    }

    #[test]
    fn test_go_forward() {
        let mut history = HistoryStore::new();
        let home = PathBuf::from("C:\\Users\\rayen");
        let docs = PathBuf::from("C:\\Users\\rayen\\Documents");

        history.navigate(&home, docs.clone());

        // Go back
        let _ = history.go_back(&docs);

        // Go forward
        let fwd = history.go_forward(&home).unwrap();
        assert_eq!(fwd, docs);
    }

    #[test]
    fn test_navigate_clears_forward() {
        let mut history = HistoryStore::new();
        let a = PathBuf::from("A");
        let b = PathBuf::from("B");
        let c = PathBuf::from("C");

        history.navigate(&a, b.clone());
        let _ = history.go_back(&b);

        // Now navigate somewhere new — forward should be cleared
        history.navigate(&a, c);
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_recent_folders() {
        let mut history = HistoryStore::new();
        let a = PathBuf::from("A");
        let b = PathBuf::from("B");
        let c = PathBuf::from("C");

        history.navigate(&a, b.clone());
        history.navigate(&b, c.clone());

        assert_eq!(history.recent[0], c);
        assert_eq!(history.recent[1], b);
    }

    #[test]
    fn test_recent_deduplication() {
        let mut history = HistoryStore::new();
        let a = PathBuf::from("A");
        let b = PathBuf::from("B");

        history.navigate(&a, b.clone());
        history.navigate(&b, a.clone());
        // Navigate back to B — should not duplicate
        history.navigate(&a, b.clone());

        // B should appear only once, at the front
        assert_eq!(history.recent.iter().filter(|p| **p == b).count(), 1);
        assert_eq!(history.recent[0], b);
    }
}

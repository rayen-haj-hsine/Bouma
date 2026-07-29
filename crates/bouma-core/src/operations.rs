//! Types for tracking file operation progress.

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// The kind of file operation being performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationKind {
    Copy,
    Move,
    Delete,
}

impl OperationKind {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            OperationKind::Copy => "Copying",
            OperationKind::Move => "Moving",
            OperationKind::Delete => "Deleting",
        }
    }
}

/// Progress of an ongoing file operation.
///
/// This is sent from the filesystem layer to the UI via channels,
/// enabling the Transparency Panel to display real-time progress.
#[derive(Debug, Clone)]
pub struct OperationProgress {
    /// What kind of operation.
    pub kind: OperationKind,

    /// Source path being operated on.
    pub source: PathBuf,

    /// Destination path (for copy/move). `None` for delete.
    pub destination: Option<PathBuf>,

    /// Total bytes to process.
    pub total_bytes: u64,

    /// Bytes processed so far.
    pub bytes_done: u64,

    /// Total number of items (files + dirs).
    pub total_items: u64,

    /// Items processed so far.
    pub items_done: u64,
}

impl OperationProgress {
    /// Returns the completion ratio as a float in `[0.0, 1.0]`.
    pub fn fraction(&self) -> f32 {
        if self.total_bytes == 0 {
            return if self.items_done >= self.total_items {
                1.0
            } else {
                0.0
            };
        }
        (self.bytes_done as f64 / self.total_bytes as f64) as f32
    }

    /// Returns the percentage complete as an integer `[0, 100]`.
    pub fn percent(&self) -> u8 {
        (self.fraction() * 100.0).min(100.0) as u8
    }
}

/// Timing diagnostics for an operation, used by the Transparency System.
///
/// Records how long each phase of an operation took so the user can
/// see exactly where time was spent.
#[derive(Debug, Clone)]
pub struct OperationDiagnostics {
    /// Label for this diagnostic (e.g., "Folder loading").
    pub label: String,

    /// Breakdown of time spent per phase.
    pub phases: Vec<(String, Duration)>,

    /// When the operation started.
    pub started_at: Instant,
}

impl OperationDiagnostics {
    /// Creates a new diagnostics tracker.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            phases: Vec::new(),
            started_at: Instant::now(),
        }
    }

    /// Records a completed phase with its duration.
    pub fn record_phase(&mut self, name: impl Into<String>, duration: Duration) {
        self.phases.push((name.into(), duration));
    }

    /// Returns the total elapsed time since the operation started.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Returns the total time across all recorded phases.
    pub fn total_phase_time(&self) -> Duration {
        self.phases.iter().map(|(_, d)| *d).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_fraction() {
        let progress = OperationProgress {
            kind: OperationKind::Copy,
            source: PathBuf::from("C:\\src"),
            destination: Some(PathBuf::from("C:\\dst")),
            total_bytes: 1000,
            bytes_done: 500,
            total_items: 10,
            items_done: 5,
        };
        assert!((progress.fraction() - 0.5).abs() < f32::EPSILON);
        assert_eq!(progress.percent(), 50);
    }

    #[test]
    fn test_progress_zero_bytes() {
        let progress = OperationProgress {
            kind: OperationKind::Delete,
            source: PathBuf::from("C:\\file"),
            destination: None,
            total_bytes: 0,
            bytes_done: 0,
            total_items: 5,
            items_done: 3,
        };
        assert!((progress.fraction() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_diagnostics() {
        let mut diag = OperationDiagnostics::new("Test operation");
        diag.record_phase("Phase 1", Duration::from_millis(100));
        diag.record_phase("Phase 2", Duration::from_millis(200));

        assert_eq!(diag.phases.len(), 2);
        assert_eq!(diag.total_phase_time(), Duration::from_millis(300));
    }
}

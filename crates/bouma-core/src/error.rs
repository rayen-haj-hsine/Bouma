//! Unified error types for Bouma.
//!
//! All crates in the workspace return `BoumaError` (or `Result<T, BoumaError>`)
//! to ensure consistent error handling across the application.

use std::path::PathBuf;
use thiserror::Error;

/// The unified error type for all Bouma operations.
#[derive(Debug, Error)]
pub enum BoumaError {
    /// An I/O error occurred during a filesystem operation.
    #[error("I/O error at {path}: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },

    /// The specified path was not found.
    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    /// Permission was denied for the specified path.
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    /// The path is not a directory (but a directory was expected).
    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),

    /// An operation was cancelled by the user.
    #[error("Operation cancelled")]
    Cancelled,

    /// A file or directory already exists at the target path.
    #[error("Already exists: {0}")]
    AlreadyExists(PathBuf),

    /// An error occurred while serializing or deserializing data.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// A search query could not be parsed.
    #[error("Invalid search query: {0}")]
    InvalidQuery(String),
}

impl BoumaError {
    /// Creates an `Io` error with the given path context.
    pub fn io(source: std::io::Error, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        // Promote specific I/O error kinds to more specific Bouma errors.
        match source.kind() {
            std::io::ErrorKind::NotFound => BoumaError::NotFound(path),
            std::io::ErrorKind::PermissionDenied => BoumaError::PermissionDenied(path),
            std::io::ErrorKind::AlreadyExists => BoumaError::AlreadyExists(path),
            _ => BoumaError::Io { source, path },
        }
    }
}

/// Convenience type alias for `Result<T, BoumaError>`.
pub type BoumaResult<T> = Result<T, BoumaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_promotion_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = BoumaError::io(io_err, "C:\\missing.txt");
        assert!(matches!(err, BoumaError::NotFound(_)));
    }

    #[test]
    fn test_io_error_promotion_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = BoumaError::io(io_err, "C:\\System");
        assert!(matches!(err, BoumaError::PermissionDenied(_)));
    }

    #[test]
    fn test_io_error_generic() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "disk error");
        let err = BoumaError::io(io_err, "C:\\data");
        assert!(matches!(err, BoumaError::Io { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = BoumaError::NotFound(PathBuf::from("C:\\gone.txt"));
        assert_eq!(err.to_string(), "Path not found: C:\\gone.txt");
    }
}

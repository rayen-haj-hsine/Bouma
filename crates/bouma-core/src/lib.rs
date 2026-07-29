//! # bouma-core
//!
//! Core domain types, traits, and business logic for the Bouma file manager.
//!
//! This crate defines the canonical representations of files, directories,
//! and operations. It has **no dependency** on any UI framework or platform-specific
//! API — it is the pure domain layer.

pub mod entry;
pub mod error;
pub mod operations;
pub mod sort;

pub use entry::{EntryKind, FileEntry, FileTypeFilter};
pub use error::BoumaError;
pub use sort::{SortField, SortOrder};

//! # bouma-search
//!
//! Fast local filename search for the Bouma file manager.
//!
//! MVP scope: searches within a provided list of `FileEntry` items.
//! No content indexing, no recursive search, no cloud.

pub mod engine;
pub mod query;

pub use engine::{search, search_scored};
pub use query::SearchQuery;

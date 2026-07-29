//! # bouma-filesystem
//!
//! Concrete filesystem operations for the Bouma file manager.
//!
//! This crate provides directory reading with parallel metadata collection,
//! and will later include file operations (copy, move, delete) with progress
//! reporting.

pub mod dir_reader;

pub use dir_reader::read_directory;

//! Dark theme for Bouma.
//!
//! A carefully chosen color palette that's easy on the eyes for
//! extended file browsing sessions.

use iced::Color;

// ── Background layers ────────────────────────────────────────────

/// The deepest background (window).
pub const BG_BASE: Color = Color::from_rgb(0.09, 0.09, 0.12);

/// Sidebar and panel backgrounds.
pub const BG_SURFACE: Color = Color::from_rgb(0.12, 0.12, 0.16);

/// Elevated surfaces (hover states, cards).
pub const BG_ELEVATED: Color = Color::from_rgb(0.16, 0.16, 0.20);

/// Active/selected item background.
pub const BG_SELECTED: Color = Color::from_rgb(0.20, 0.25, 0.42);

// ── Text ─────────────────────────────────────────────────────────

/// Primary text (filenames, headings).
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.93, 0.93, 0.96);

/// Secondary text (metadata, sizes, dates).
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.60, 0.66);

/// Muted text (disabled, placeholders).
pub const TEXT_MUTED: Color = Color::from_rgb(0.40, 0.40, 0.45);

// ── Accent ───────────────────────────────────────────────────────

/// Primary accent (links, active elements).
pub const ACCENT: Color = Color::from_rgb(0.40, 0.58, 1.0);

/// Accent hover state.
pub const ACCENT_HOVER: Color = Color::from_rgb(0.50, 0.66, 1.0);

// ── Semantic ─────────────────────────────────────────────────────

/// Success / positive actions.
pub const SUCCESS: Color = Color::from_rgb(0.30, 0.78, 0.48);

/// Warning indicators.
pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.25);

/// Error / destructive actions.
pub const ERROR: Color = Color::from_rgb(0.90, 0.30, 0.30);

// ── Borders ──────────────────────────────────────────────────────

/// Subtle borders and dividers.
pub const BORDER: Color = Color::from_rgb(0.20, 0.20, 0.25);

// ── Spacing constants ────────────────────────────────────────────

/// Standard padding inside containers.
pub const PADDING: f32 = 12.0;

/// Small padding for tight spaces.
pub const PADDING_SM: f32 = 6.0;

/// Large padding for sections.
pub const PADDING_LG: f32 = 20.0;

/// Standard spacing between items.
pub const SPACING: f32 = 8.0;

/// Small spacing.
pub const SPACING_SM: f32 = 4.0;

/// Height of a single file row.
pub const ROW_HEIGHT: f32 = 32.0;

/// Sidebar default width.
pub const SIDEBAR_WIDTH: f32 = 220.0;

/// Toolbar height.
pub const TOOLBAR_HEIGHT: f32 = 44.0;

/// Status bar height.
pub const STATUS_BAR_HEIGHT: f32 = 28.0;

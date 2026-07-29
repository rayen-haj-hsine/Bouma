//! Bouma — A minimal, fast, offline-first file manager.
//!
//! Application entry point.

mod app;
mod message;
mod theme;
mod views;

use app::Bouma;
use tracing_subscriber::EnvFilter;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Bouma v{}", env!("CARGO_PKG_VERSION"));

    iced::application(Bouma::new, Bouma::update, Bouma::view)
        .title(Bouma::title)
        .theme(Bouma::theme)
        .window_size(iced::Size::new(1100.0, 700.0))
        .run()
}

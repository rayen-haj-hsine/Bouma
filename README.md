# 🦉 Bouma

**A minimal, fast, offline-first file manager for Windows.**

> *Bouma* — see your files clearly, nothing more.

[![CI](https://github.com/rayen/bouma/actions/workflows/ci.yml/badge.svg)](https://github.com/rayen/bouma/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Philosophy

- **Minimal** — No bloat. Every feature earns its place.
- **Snappy** — Folders open instantly. No spinners, no waiting.
- **Accurate** — What you see is what's on disk. Always.
- **Offline-first** — Zero network connections. Zero telemetry. Zero cloud.
- **Transparent** — You always know what Bouma is doing and why.

## Features (MVP)

- ⚡ Fast file browsing with parallel directory reading
- 📁 File operations (copy, move, delete, rename) with progress
- 🔍 Fast local search (filename, extension, date filtering)
- 🔬 Transparency panel — see operation speed, diagnostics, ETA
- 🌙 Dark mode UI
- 🔒 Fully offline — no network, no telemetry, no external services
- 🦀 Built entirely in Rust

## Architecture

Bouma is organized as a Cargo workspace with clean separation of concerns:

```
crates/
├── bouma-app          # Iced GUI application
├── bouma-core         # Domain types, traits, business logic
├── bouma-filesystem   # Directory reading, file operations
├── bouma-search       # Filename search engine
└── bouma-cache        # Settings, history, metadata cache
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (stable, latest)
- Windows 10 or later

### Build & Run

```bash
# Debug build
cargo run -p bouma-app

# Release build (optimized)
cargo build --release
```

### Development

```bash
# Run tests
cargo test --all

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets -- -D warnings
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full development plan.

## License

[MIT](LICENSE)

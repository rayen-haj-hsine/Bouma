# 🌲 Bouma

**Bouma** is a high-performance, offline-first, modern file navigation and search system built with Rust and [Iced](https://github.com/iced-rs/iced). Designed as a lightweight, intelligent alternative to traditional desktop file explorers, Bouma prioritizes speed, clarity, and visual system layout.

---

## ✨ Features

- **🗺️ Interactive System Mind Map**: Visualizes your drive hierarchy as a dynamic, tree-structured mind map starting from drive root (`C:/`).
- **🌳 Tree Expansion & Branch Pruning**:
  - Expand subfolders directly within the Mind Map view.
  - Mark heavy or irrelevant folders as **Closed** to prune them entirely from recursive searches.
- **⚡ High-Speed Parallel Search**:
  - Multi-threaded disk traversal using `jwalk` and `Rayon`.
  - Manual execution on `Enter` to prevent unnecessary IO churn.
- **🎯 Smart Categorization & Relevance Tiering**:
  - Categorizes search results into **Exact**, **Prefix**, **Word**, and **Partial** tiers.
  - File extension labels (e.g., `Documents (.pdf, .doc, .txt)`).
  - Exact file & extension matching (e.g. `id.txt`).
- **🔍 Transparency & Diagnostics Panel**: Real-time feedback showing items scanned, active search status (`⟳ Searching…`), traversal timing, and tier distribution.

---

## 🛠️ Architecture

Bouma is structured as a modular Rust workspace:

```text
Bouma/
├── crates/
│   ├── bouma-app/         # GUI implementation using Iced (Elm architecture)
│   ├── bouma-core/        # Domain entities, sorting algorithms, file types
│   ├── bouma-filesystem/  # Multi-threaded filesystem walker & operations
│   ├── bouma-search/      # Search query parser & relevance tiering engine
│   └── bouma-cache/       # User settings & navigation history tracking
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or newer)
- Cargo

### Installation & Execution

Clone the repository and run via Cargo:

```bash
git clone https://github.com/rayen-haj-hsine/Bouma.git
cd Bouma
cargo run --bin bouma --release
```

### Running Tests

Run the full workspace unit and integration test suite:

```bash
cargo test --all
```

---

## ⌨️ Hotkeys & Controls

- **`Enter`**: Run search query.
- **`Esc` / `Clear (✕)`**: Clear current search text and filter while preserving active directory.
- **`🔒 Close & Prune`**: Prune directory from recursive search traversal.
- **`▶` / `▼`**: Expand or collapse subfolder branches in Mind Map view.

---

## 📄 License

Dual-licensed under MIT or Apache 2.0.

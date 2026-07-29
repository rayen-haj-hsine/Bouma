# Bouma — Architecture

## Overview

Bouma is a Rust application organized as a Cargo workspace. It follows a
**hexagonal architecture** (ports & adapters) where the core business logic
has no knowledge of the UI framework or platform-specific APIs.

## Crate Dependency Graph

```
bouma-app (binary)
├── bouma-core
├── bouma-filesystem
│   └── bouma-core
├── bouma-search
│   └── bouma-core
└── bouma-cache
    └── bouma-core
```

`bouma-core` is the leaf dependency — it defines domain types and traits
that all other crates depend on, but it depends on nothing project-internal.

## Crate Responsibilities

### bouma-core

**Domain types, traits, and business logic.**

- `FileEntry` — the canonical representation of a file/directory
- `EntryKind` — enum: File, Directory, Symlink
- `SortField`, `SortOrder` — sorting configuration
- `BoumaError` — unified error type
- `DirectoryReader` trait — port for filesystem access (enables mocking)
- Sorting, filtering, and comparison logic

### bouma-filesystem

**Concrete filesystem operations.**

- `NativeDirectoryReader` — implements `DirectoryReader` using `std::fs`
- Parallel metadata collection via `rayon`
- Recursive traversal via `jwalk`
- File operations (copy, move, delete, rename) with progress channels
- Streaming support for large directories via `tokio::sync::mpsc`

### bouma-search

**Filename search engine.**

- `SearchQuery` — parsed query model
- `SearchEngine` — filters entries by name, extension, date range
- Glob pattern matching via `globset`
- No content indexing (MVP)

### bouma-cache

**Local persistence.**

- Settings storage (`%APPDATA%/Bouma/settings.json`)
- Folder history (recently visited)
- Favorites (pinned folders)
- Future: thumbnail cache, SQLite metadata index

### bouma-app

**Iced GUI application.**

Follows the **Elm architecture** (Model → Update → View):

```
┌─────────────┐     ┌──────────┐     ┌──────────┐
│   Message    │────▶│  update() │────▶│  State   │
│  (user input,│     │  (logic)  │     │ (model)  │
│   async task)│     └──────────┘     └────┬─────┘
└─────────────┘                            │
       ▲                                   ▼
       │                            ┌──────────┐
       └────────────────────────────│  view()   │
                                    │  (render) │
                                    └──────────┘
```

UI components:
- Sidebar — drives, favorites, quick access
- File list — virtualized scrollable table
- Breadcrumb — path navigation
- Toolbar — back/forward, search, view toggle
- Transparency panel — operation progress + diagnostics
- Status bar — selection info, item count

## Data Flow: Opening a Directory

```
1. User double-clicks folder
2. → Message::OpenDirectory(path)
3. → update() pushes current path to history
4. → update() spawns async Task:
       filesystem.read_dir(path)
5. → UI shows loading indicator
6. → Task completes
7. → Message::DirectoryLoaded(Result<Vec<FileEntry>>)
8. → update() stores entries, clears loading state
9. → view() renders file list with new entries
```

## Design Principles

1. **No UI in core** — `bouma-core` never imports `iced`
2. **Traits as ports** — filesystem access is behind traits for testability
3. **Messages, not callbacks** — all state changes go through the message system
4. **Errors are values** — no panics in normal operation, `Result` everywhere
5. **Zero network** — no crate may open network connections

# Bouma — Roadmap

## Status Legend

- ✅ Done
- 🚧 In Progress
- ⬚ Planned

---

## Phase 0 — Project Foundation 🚧

- [x] Define project name, mission, and principles
- [x] Create repository structure
- [x] Write initial documentation (README, ARCHITECTURE, ROADMAP)
- [x] Set up CI/CD pipeline

## Phase 1 — Technical Foundation 🚧

- [x] Choose stack (Rust + Iced + jwalk)
- [x] Create Cargo workspace with 5 crates
- [ ] Implement core domain types
- [ ] Set up error handling
- [ ] Configure logging

## Phase 2 — Filesystem Engine ⬚

- [ ] Directory reading with metadata
- [ ] Parallel metadata collection (rayon)
- [ ] Streaming results for large directories
- [ ] Lazy loading support
- [ ] Performance benchmarks

## Phase 3 — Basic User Interface ⬚

- [ ] Main layout (sidebar + file list)
- [ ] Sidebar (drives, favorites)
- [ ] File list with columns
- [ ] Breadcrumb navigation
- [ ] Back/forward navigation
- [ ] Status bar

## Phase 4 — File Operations ⬚

- [ ] Open file/folder
- [ ] Rename
- [ ] Delete (to trash)
- [ ] Create folder
- [ ] Copy with progress
- [ ] Move with progress
- [ ] Cancellation support

## Phase 5 — Transparency System ⬚

- [ ] Operation progress panel
- [ ] Speed and ETA display
- [ ] Diagnostic timing breakdown
- [ ] Error display

## Phase 6 — Search ⬚

- [ ] Current folder search
- [ ] Filename matching
- [ ] Extension filtering
- [ ] Date filtering
- [ ] Sort results

## Phase 7 — Local Cache ⬚

- [ ] Settings persistence
- [ ] Folder history
- [ ] Thumbnail cache
- [ ] Metadata cache

## Phase 8 — Quality ⬚

- [ ] Comprehensive error handling
- [ ] Crash protection
- [ ] Structured logging
- [ ] Performance benchmarks
- [ ] Memory optimization

## Phase 9 — UI Polish ⬚

- [ ] Dark theme refinement
- [ ] File type icons
- [ ] Smooth animations
- [ ] Keyboard shortcuts
- [ ] Responsive layout

## Phase 10 — Testing ⬚

- [ ] Unit tests (all crates)
- [ ] Integration tests
- [ ] Manual testing scenarios

## Phase 11 — Release ⬚

- [ ] Finalize documentation
- [ ] Screenshots and demo
- [ ] Windows installer (MSI)
- [ ] Portable executable
- [ ] v0.1.0 release

---

## Post-MVP (Future)

- Tabs
- Split view
- Preview pane
- Duplicate finder
- Disk analyzer
- Git integration
- Plugin system
- Linux/macOS support

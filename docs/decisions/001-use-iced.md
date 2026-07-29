# ADR-001: Use Iced for GUI Framework

## Status

Accepted

## Context

Bouma needs a GUI framework for its desktop file manager interface. The candidates
evaluated were:

- **Iced** — Pure Rust, Elm architecture, MIT licensed
- **Slint** — Rust + `.slint` DSL, triple-licensed (Royalty-free/GPLv3/Commercial)
- **egui** — Immediate mode, MIT licensed

## Decision

Use **Iced 0.14** as the GUI framework.

## Rationale

1. **Pure Rust** — No DSL files, no mixed toolchains
2. **MIT License** — No attribution requirements, no copyleft concerns
3. **Elm Architecture** — Message-driven pattern maps naturally to file manager interactions
4. **Reactive Rendering** — Near-zero CPU when idle (critical for an app that stays open all day)
5. **Comet Debugger** — Time-travel debugging accelerates development
6. **Growing Ecosystem** — `iced_aw` for additional widgets, active community

## Consequences

- We must build custom widgets for context menus and virtualized file lists
- This gives us complete control over the UX
- No vendor lock-in or licensing concerns

# Changelog

## 1.0.0 - 2026-11-05

- Fork the upstream GPLv3 terminal game into `kpihx-labs/2048` while preserving history and attribution.
- Replace fixed `5x1` rendering with responsive pane-filling geometry.
- Add `auto`, `compact`, `balanced`, `large`, and manually dimensioned layouts.
- Redraw immediately after terminal resize events.
- Render large block numerals when tile dimensions permit them.
- Install a native binary named `2048` through Cargo instead of Snap.
- Move score persistence to the XDG data directory with non-destructive legacy migration.
- Add CLI, layout, display, and row-integrity tests alongside the original game-logic suite.
- Add one strict local `make check` quality gate without redundant remote CI.

## Upstream baseline

- Original Rust and Crossterm implementation by Ken VanDine and upstream contributors.
- Color themes, 2048 game logic, keyboard controls, and top-ten score tracking.

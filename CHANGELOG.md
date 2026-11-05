# Changelog

## 1.1.0 - 2026-11-05

- Freeze the final board at game over and draw a visible `GAME OVER` overlay instead of clearing the result immediately.
- Add explicit restart, history, and quit actions on the frozen result.
- Persist the final 4x4 board with each ranked performance while keeping legacy score files readable.
- Make performance history selectable with arrows, page keys, mouse wheel, and direct row clicks.
- Reconstruct each stored final board responsively and browse snapshots without leaving history.
- Enable mouse capture only for the game session and restore terminal state on exit.

## 1.0.1 - 2026-11-05

- Replace the hand-authored three-row digit matrices with the maintained `tui-big-text` renderer and its tested 8x8 font.
- Select full-pixel or Unicode-quadrant glyphs automatically from the available tile geometry, with clipping-safe plain-number fallback.
- Cover `2`, `128`, `1024`, and `2048` across split-pane and tall-tile rendering tests.
- Derive CLI version output directly from the Cargo package version.

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

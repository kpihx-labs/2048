# AGENTS.md

This repository is the KπX Labs fork of **terminal-2048**. It owns a native,
responsive `2048` binary while preserving the upstream GPLv3 history and
attribution.

## Architecture

- `src/game/logic.rs` owns game state and moves.
- `src/game/layout.rs` owns pure terminal geometry.
- `src/game/display.rs` owns adaptive number labels.
- `src/game/ui.rs` owns terminal lifecycle, input, and rendering.
- `src/cli.rs` owns the public command-line contract.
- `CONTRACT.md` is the behavioral source of truth.

## Tests

Run `make check`. New geometry or CLI behavior requires focused unit tests. Keep
the original integration tests for movement rules. `tests/suite/` remains the
optional real-terminal visual smoke layer.

## Conventions

- Preserve the upstream GPLv3 license and provenance.
- Keep `auto` responsive and presets deterministic.
- Never hardcode a home path; use platform/XDG directory APIs.
- Install locally through `make install` and push through `make push`.
- Do not reintroduce Snap packaging in this fork.

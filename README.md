# 2048

A colorful, responsive 2048 game that treats the terminal as a real interface rather than a fixed text canvas. The board grows with a tmux pane, redraws when the pane is resized, and can also use deterministic tile dimensions.

This repository is a GPLv3 fork of [Ken VanDine's terminal-2048](https://github.com/kenvandine/terminal-2048). The original game logic, score system, history, and authorship remain preserved.

## What changed in this fork

- Responsive layout fills the current terminal or tmux pane.
- Live redraw on terminal resize.
- Large block numerals when tiles have enough room.
- Five layout modes: `auto`, `compact`, `balanced`, `large`, and `manual`.
- Explicit manual tile width and height.
- Native `2048` binary, independent of Snap.
- XDG-compliant score storage with migration from the legacy score file.
- Strict local formatting, Clippy, unit, integration, release-build, and CLI smoke checks through one `make check` gate.

## Install

### From this checkout

```bash
git clone git@github.com:kpihx-labs/2048.git
cd 2048
make install
```

`make install` runs the complete check suite and installs the native `2048` binary through Cargo.

### Build without installing

```bash
make build
./target/release/2048
```

## Play

```bash
2048
```

The default `auto` mode uses all safe space available in the current terminal. A `118x30` split pane, for example, produces four `28x4` tiles and a `117x21` board.

### Layout modes

```bash
2048 --layout auto
2048 --layout compact
2048 --layout balanced
2048 --layout large
2048 --layout manual --tile-width 18 --tile-height 5
```

| Mode | Behavior |
|---|---|
| `auto` | Recalculates tile dimensions from the current terminal size. |
| `compact` | Fixed `5x1` tiles for very small panes. |
| `balanced` | Fixed `11x3` tiles with large numerals. |
| `large` | Fixed `17x5` tiles for roomy terminals. |
| `manual` | Requires both `--tile-width` and `--tile-height`. |

Run `2048 --help` for the complete CLI contract.

## Controls

| Key | Action |
|---|---|
| Arrow keys or `WASD` | Move the tiles |
| `H` | Show high scores |
| `R` | Restart the board |
| `Q` or `Esc` | Quit |

## Scores

Scores live at:

```text
$XDG_DATA_HOME/2048/high-scores.json
```

When `XDG_DATA_HOME` is unset, the standard location is `$HOME/.local/share/2048/high-scores.json`. Existing `$HOME/.2048_high_scores.json` data is read and migrated without deleting the legacy file.

## Development

```bash
make help
make check
```

The render geometry is pure and tested independently from the terminal backend. Tests cover auto growth, split-pane dimensions, manual dimensions, minimum-size errors, CLI validation, large numerals, tile row integrity, and the original movement rules. `make check` is the sole quality gate; no redundant remote CI duplicates it.

Maintainers finalize a version with `make release`, which checks the project and synchronizes the branch and Cargo-derived release tag to both mirrors.

## Architecture

- `src/cli.rs`: command-line contract and validation.
- `src/game/layout.rs`: terminal-to-board geometry.
- `src/game/display.rs`: adaptive plain and block numeral labels.
- `src/game/ui.rs`: terminal lifecycle, events, and rendering.
- `src/game/logic.rs`: 2048 movement and merge rules.
- `src/scores.rs`: score persistence and legacy migration.

The behavioral contract is specified in [`CONTRACT.md`](CONTRACT.md).

## License and provenance

GNU GPL v3 or later. See [`LICENSE`](LICENSE). This fork preserves the upstream license, commit history, and attribution.
